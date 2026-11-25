PR21-Wiedergabe/GPS-Test für Ubuntu 24.04 x86_64

Zuerst die vorige PR21-Testversion schließen. Die vorhandenen Videokopien in
~/Downloads/TripViewer-PR21-Testarchiv-5dbdaf9 werden weiterverwendet.
Es ist kein erneutes Kopieren des Archivs notwendig. Kein produktives Archiv
öffnen. Deine installierte 0.13.0 bleibt unverändert.

Nach dem Entpacken im Paketordner:
  sha256sum -c SHA256SUMS.txt
  ./start-test.sh

Eigenes Einstellungsverzeichnis:
  ~/.local/share/tripviewer-pr21-test/680a9b2
Fenstertitel: Trip Viewer PR21 Test — 680a9b2
Native Testversion: 0.3.4-rc.2026112504. Die unveränderte Oberfläche kann
weiterhin die Upstream-Paketversion 0.3.3 anzeigen.

Die Hauptdatenbank bleibt auf Upstream-Schema 13. Zwischenergebnisse und
Medienfehler werden separat in .tripviewer/analysis-cache-v1.sqlite gespeichert.
Es gibt keine zusätzlichen Upstream-Migrationen. Dieser Cache ist aus den
Quelldateien wiederherstellbar und liegt beim Testarchiv, nicht bei 0.13.0.

Kleiner Test: eine Fahrt auswählen, Original und Geschwindigkeit x1 einstellen.
Ton prüfen; andere Geschwindigkeiten und Timelapse-Modi sind absichtlich stumm.
Frontvideo und weitere Kanäle kurz zusammen abspielen, pausieren und springen.
GPS-Fortschritt erscheint als abgeschlossene Segmente; Karte und Geschwindigkeit
sollen bereits während der Hintergrundverarbeitung sichtbar werden.
Eine vollständige Neuberechnung aller Fahrten ist für diese erste Prüfung nicht
nötig. Keine Parkaufnahmen oder neue Timelapse-Codierung erforderlich.

Die vorhandenen Linux-Medienkorrekturen, stabile Synchronisation, kanalweise
Fehlerbehandlung und segmentweise GPS-Zwischenergebnisse stammen aus dem
vollständigen Fork. Linux libva/DMABuf-Arbeit von @gubiq bleibt gewürdigt;
ursprüngliche Commit-Autorschaft und Lizenzhinweise bleiben erhalten.
Codex hat die gezielte Übernahme und Paketierung vorbereitet. Der Build- und
Starttest ersetzt keinen Hardwaretest mit deinen Aufnahmen. Eine Behebung der
beobachteten Symptome ist erst nach diesem Test bestätigt.

Eigene App-Kennung com.tripviewer.pr21test, deaktivierte Update-Endpunkte.
Die Drei-Sekunden-Dateizuordnung bleibt eine Heuristik; in diesem Paket wurde
keine GPS-basierte Kanalzuordnung eingeführt. Keine automatischen Log-Uploads.
Bei Fehlern reicht zunächst eine kurze Beschreibung; keine Rohvideos oder
GPS-Rohdaten veröffentlichen.

Gezielter GPS-Nachtest:
  TRIPVIEWER_GPS_TIMING=1 ./start-test.sh
Die Ausgabe erscheint jetzt im Terminal und in last-run.log. Eine vorhandene
Fahrt auswählen; Videos dürfen pausiert bleiben. GPS-Reader-Version9 prüft
die bisherigen Version8-Zwischenstände einmal neu, ohne sie manuell zu löschen.
Die bisherige 4-KB-Grenze für ganze GPS-Container ist durch einen begrenzten
Lesepuffer ersetzt. Dass dies die Ursache der echten Aufnahmen war, ist noch
nicht bestätigt. Bei Lesefehlern wird ein Fehler statt eines erfolgreichen
leeren Tracks gemeldet. Medienfehler, Ruckeln und Vollbild-Synchronität sind
weiterhin offen; dieses Paket ist keine Abnahme dieser Probleme.
