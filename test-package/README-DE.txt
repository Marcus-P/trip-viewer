Isolierter PR21-Test für Ubuntu 24.04 x86_64

Quelle: work/viofo-pr21-reliability, Commit 5dbdaf9.
PR21 plus lokale GPS-, Gruppierungs- und RO/Import-Korrekturen.
Kein Update des installierten Forks v0.13.0. Keine Installation notwendig.
Codex hat Build und Testpaket vorbereitet. Noch kein Test mit deinen Videos.

Voraussetzung: Ubuntu 24.04 x86_64, grafische Sitzung, bash und python3.
Nach dem Entpacken:
  sha256sum -c SHA256SUMS.txt
  ./start-test.sh

Eigenes Testarchiv:
  ~/Downloads/TripViewer-PR21-Testarchiv-5dbdaf9
Eigene Einstellungen und lokales Startlog:
  ~/.local/share/tripviewer-pr21-test/5dbdaf9

Kopiere vor dem Start wenige vollständige Kanalsets zweier aufeinanderfolgender
Segmente nach Videos im Testarchiv. Nur Kopien verwenden. Geschützte Aufnahmen
gehören in Videos/RO; vorhandene Dateinamen unverändert lassen. Falls verfügbar,
kannst du zusätzlich ein vollständiges Parkmodus-Kanalset verwenden.
Öffne in dieser Testversion kein produktives Archiv. Der Starthelfer setzt bei
jedem Start das eigene Testarchiv. Die App erlaubt dennoch manuelle Archivwechsel.

Kleiner Test: Kanalzuordnung, Wiedergabe/Pause, Springen und Segmentübergang.
Bleiben die Kanäle synchron? Werden RO-Clips als Ereignis erkannt? Parkmodus und
Normalmodus sollen separate Segmente bleiben, dürfen aber derselben Fahrt
angehören. Es ist keine neue Timelapse-Codierung erforderlich.

Der Fenstertitel enthält PR21 Test und 5dbdaf9. Die Oberfläche kann weiterhin
0.3.3 anzeigen, weil sie die unveränderte Upstream-Paketversion verwendet.
Das native Testpaket hat Version 0.3.4-rc.20261125 und eine eigene App-Kennung.
Der Test enthält nur den PR21-Umfang samt Korrekturen, nicht alle Funktionen
von Fork v0.13.0. Automatische Update-Endpunkte sind im Testpaket deaktiviert.
Es gibt keinen automatischen Upload des Startlogs.

Die Drei-Sekunden-Toleranz vergleicht Aufnahmezeitstempel vorhandener Dateien.
Es gibt hierfür kein Warten auf weitere Dateien. Die Zuordnung bleibt eine
begrenzte Heuristik und beweist kein gemeinsames physisches Aufnahmeereignis.

Eine kurze Rückmeldung reicht. Keine Videos, GPS-Rohdaten oder vollständigen
Logs veröffentlichen. Falls ein Fehler auftritt, zuerst die sichtbare Meldung
und den betroffenen Testschritt mitteilen.
