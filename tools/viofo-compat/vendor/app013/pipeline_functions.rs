fn batch_quality(item: &BatchGps, duration_s: f64) -> (usize, i64) {
    let offsets: Vec<f64> = match item {
        BatchGps::Plain(item) => item
            .points
            .iter()
            .filter(|p| p.fix_ok && p.t_offset_s >= 0.0 && p.t_offset_s <= duration_s)
            .map(|p| p.t_offset_s)
            .collect(),
        BatchGps::Viofo { timed, .. } => timed
            .iter()
            .filter(|p| {
                p.point.fix_ok && p.point.t_offset_s >= 0.0 && p.point.t_offset_s <= duration_s
            })
            .map(|p| p.point.t_offset_s)
            .collect(),
    };
    if offsets.is_empty() {
        return (0, 0);
    }
    let span_ms = ((offsets[offsets.len() - 1] - offsets[0]).max(0.0) * 1000.0) as i64;
    (offsets.len(), span_ms)
}

fn best_segment_gps(items: impl IntoIterator<Item = BatchGps>, duration_s: f64) -> BatchGps {
    let mut best: Option<((usize, i64), BatchGps)> = None;
    for item in items {
        let quality = batch_quality(&item, duration_s);
        if best.as_ref().map(|(q, _)| quality > *q).unwrap_or(true) {
            best = Some((quality, item));
        }
    }
    best.map(|(_, item)| item).unwrap_or_else(|| {
        BatchGps::Plain(GpsBatchItem {
            file_path: String::new(),
            points: Vec::new(),
        })
    })
}

fn trip_segment_points(item: &BatchGps, duration: f64, offset: f64) -> Vec<GpsPoint> {
    let points: Vec<&GpsPoint> = match item {
        BatchGps::Plain(item) => item.points.iter().collect(),
        BatchGps::Viofo { timed, .. } => timed.iter().map(|p| &p.point).collect(),
    };
    points
        .into_iter()
        .filter(|p| p.t_offset_s >= 0.0 && p.t_offset_s <= duration)
        .map(|p| GpsPoint {
            t_offset_s: offset + p.t_offset_s,
            ..p.clone()
        })
        .collect()
}

fn extract_trip_gps_segments<E>(
    segments: &[TripGpsSegmentSource],
    mut extract: impl FnMut(usize, &TripGpsSegmentSource) -> Result<BatchGps, E>,
    mut progress: impl FnMut(usize, Vec<TripGpsSegmentUpdate>),
) -> Result<Vec<GpsPoint>, E> {
    let mut selected: Vec<BatchGps> = Vec::with_capacity(segments.len());
    let mut offsets = Vec::with_capacity(segments.len());
    let mut cursor = 0.0;
    for (index, segment) in segments.iter().enumerate() {
        offsets.push(cursor);
        cursor += segment.duration_s;
        selected.push(extract(index, segment)?);
        let start = index.saturating_sub(1);
        validate_viofo_batch_boundaries(&mut selected[start..]);
        let updates = (start..=index)
            .map(|i| TripGpsSegmentUpdate {
                index: i,
                points: trip_segment_points(&selected[i], segments[i].duration_s, offsets[i]),
            })
            .collect();
        progress(index + 1, updates);
    }
    Ok(selected
        .iter()
        .enumerate()
        .flat_map(|(i, item)| trip_segment_points(item, segments[i].duration_s, offsets[i]))
        .collect())
}

fn extract_viofo_timed(path: &Path) -> Result<Vec<viofo::TimedGpsPoint>, AppError> {
    // The consistency validator needs the clip's measured duration only as an
    // upper bound on how far apart two UTC records from this same MP4 can be.
    // It does not use the filename clock or infer any timezone.
    let started = Instant::now();
    let duration_s = crate::metadata::mp4_probe::probe(path)?.duration_s;
    gps_timing(path, "metadata", started.elapsed(), 0);
    let mut timed = viofo::extract_timed(path, Some(duration_s))?;
    let Some(first_gps_utc) = timed.first().map(|item| item.utc) else {
        return Ok(timed);
    };

    // Synchronization with the local camera clock remains a separate concern
    // from data validation. For now keep the existing best-effort civil-offset
    // inference, but feed it the first observation that survived consistency
    // validation so malformed raw records cannot become the timing anchor.
    match viofo_timing::first_fix_delay_s(path, first_gps_utc, duration_s) {
        Some(delay_s) if delay_s > 0.0 => {
            for item in &mut timed {
                item.point.t_offset_s += delay_s;
            }
        }
        _ => {}
    }

    Ok(timed)
}

fn validate_viofo_batch_boundaries(batch: &mut [BatchGps]) {
    for index in 0..batch.len().saturating_sub(1) {
        let (left_slice, right_slice) = batch.split_at_mut(index + 1);
        let left = &mut left_slice[index];
        let right = &mut right_slice[0];

        let (
            BatchGps::Viofo {
                file_path: left_path,
                timed: left_timed,
            },
            BatchGps::Viofo {
                file_path: right_path,
                timed: right_timed,
            },
        ) = (left, right)
        else {
            continue;
        };

        validate_viofo_boundary(left_path, left_timed, right_path, right_timed);
    }
}

fn validate_viofo_boundary(
    left_path: &str,
    left: &mut Vec<viofo::TimedGpsPoint>,
    right_path: &str,
    right: &mut Vec<viofo::TimedGpsPoint>,
) {
    if left.len() < 2 || right.len() < 2 || !boundary_has_local_utc_continuity(left, right) {
        return;
    }

    let left_last = left.len() - 1;
    let points = [&left[left_last - 1], &left[left_last], &right[0], &right[1]];
    let observations: Vec<validation::GpsObservation> = points
        .iter()
        .enumerate()
        .map(|(source_order, item)| boundary_observation(item, source_order))
        .collect();
    let validated = validation::validate(&observations, validation::ValidationContext::default());

    let drop_left = !validated[1].usable_timed_position();
    let drop_right = !validated[2].usable_timed_position();

    if drop_left {
        left.pop();
        eprintln!(
            "viofo gps: boundary validation excluded terminal route point in {left_path} using UTC evidence from {right_path}"
        );
    }
    if drop_right {
        right.remove(0);
        eprintln!(
            "viofo gps: boundary validation excluded initial route point in {right_path} using UTC evidence from {left_path}"
        );
    }
}

fn boundary_observation(
    item: &viofo::TimedGpsPoint,
    source_order: usize,
) -> validation::GpsObservation {
    // A zero speed is valid data, but after the first validation pass it can
    // also be the neutral UI fallback for an unusable speed/heading field. Do
    // not use such a value as an independent cross-file witness. This only
    // weakens the boundary proof; it never rejects a route point by itself.
    let moving = item.point.speed_mps > 0.0;
    validation::GpsObservation {
        source_order,
        time_ms: Some(item.utc.and_utc().timestamp_millis()),
        latitude: Some(item.point.lat),
        longitude: Some(item.point.lon),
        speed_mps: moving.then_some(item.point.speed_mps),
        heading_deg: moving.then_some(item.point.heading_deg),
    }
}

fn boundary_has_local_utc_continuity(
    left: &[viofo::TimedGpsPoint],
    right: &[viofo::TimedGpsPoint],
) -> bool {
    let left_times: Vec<i64> = left
        .iter()
        .map(|item| item.utc.and_utc().timestamp_millis())
        .collect();
    let right_times: Vec<i64> = right
        .iter()
        .map(|item| item.utc.and_utc().timestamp_millis())
        .collect();
    boundary_times_are_locally_continuous(&left_times, &right_times)
}

fn boundary_times_are_locally_continuous(left: &[i64], right: &[i64]) -> bool {
    if left.len() < 2 || right.len() < 2 {
        return false;
    }

    let cross_gap = right[0] - left[left.len() - 1];
    if cross_gap < 0 {
        return false;
    }

    let left_gap = left
        .windows(2)
        .rev()
        .find_map(|pair| positive_time_gap(pair[0], pair[1]));
    let right_gap = right
        .windows(2)
        .find_map(|pair| positive_time_gap(pair[0], pair[1]));

    match (left_gap, right_gap) {
        (Some(left_gap), Some(right_gap)) => cross_gap <= left_gap.max(right_gap),
        _ => false,
    }
}

fn positive_time_gap(a: i64, b: i64) -> Option<i64> {
    b.checked_sub(a).filter(|gap| *gap > 0)
}
