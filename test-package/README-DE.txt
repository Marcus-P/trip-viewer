Isolierter PR21-Wiedergabetest für Ubuntu 24.04 x86_64

Vor dem Start die alte PR21-Testversion schließen. Die vorhandenen Videokopien
und manuellen Zusammenführungen im bisherigen Testarchiv werden weiterverwendet:
  ~/Downloads/TripViewer-PR21-Testarchiv-5dbdaf9
Kein produktives Archiv öffnen. Die installierte 0.13.0 bleibt unverändert.

Nach dem Entpacken im Paketordner:
  sha256sum -c SHA256SUMS.txt
  TRIPVIEWER_GPS_TIMING=1 bash ./start-test.sh

Fenstertitel: Trip Viewer PR21 Test — 17f6f97
Testversion: 0.3.4-rc.2026112507
Separate Einstellungen und Log:
  ~/.local/share/tripviewer-pr21-test/17f6f97/last-run.log
Die Ausgaben erscheinen auch direkt im Terminal.

Die Hauptdatenbank bleibt bei Schema13. Analyse- und Medienfehlerdaten stehen
separat in .tripviewer/analysis-cache-v1.sqlite. GPS-Reader-Version9 bleibt
unverändert; gültige bisherige Ergebnisse können wiederverwendet werden.
Kein Kopieren oder Löschen nötig. GPS muss nicht erneut vollständig geprüft werden.

Kleiner Wiedergabetest mit einer vorhandenen Fahrt:
1. Original und Geschwindigkeit x1. Kurz mit drei Kameras abspielen.
2. Pause/Play und ein Sprung in der Timeline. Kanalpositionen vergleichen.
3. Front doppelklicken. Im Vollbild denselben gemeinsamen Play/Pause-Knopf
   verwenden. Per Doppelklick oder Escape zurück und Ausrichtung vergleichen.

Der Browser vergrößert nun den gemeinsamen Player einschließlich Bedienung,
nicht das einzelne native Video. Kameraelemente bleiben erhalten. Das fehlende
GL-Plugin ist jetzt explizit enthalten und seine Fabriken werden vor der
Veröffentlichung geprüft. Das ist kein Nachweis einer flüssigen 4K-Wiedergabe
auf realer Hardware; diese Abnahme steht für diese Revision noch aus.
GPS-Tracks und der bewegte Marker wurden bereits im vorigen GPS-Paket bestätigt.
Ein erneuter vollständiger GPS-Test ist für diese Wiedergabekorrektur nicht nötig.

Anleitung für selbst gebaute Windows- und Ubuntu-Versionen:
  docs/VIOFO_TESTING.md im Quellcode des PR-Zweigs contrib/viofo-camera-only
Die Testkennung com.tripviewer.pr21test und deaktivierte Update-Endpunkte
verhindern eine Aktualisierung der produktiven Installation durch dieses Paket.
Einstellungen, km/h, Kartennachführung, Overlays und Bookmarks sind nicht Teil
dieses Wiedergabenachtests. Die Drei-Sekunden-Dateizuordnung bleibt heuristisch.

Lizenz- und Herkunftshinweise bleiben erhalten, einschließlich @gubiqs
Linux-Korrektur und der synthetischen A129-Layoutregression nach @alinnastac.
Codex unterstützt die Umsetzung; Hardwaretests sind davon getrennte Befunde.
Logs können private Pfade enthalten; nicht ungeprüft öffentlich posten.

Timelapse-Korrektur:
Die vorhandene NVIDIA/CPU-Auswahl prüft nun den tatsächlichen NVENC-Pfad.
Bei fehlender nutzbarer NVIDIA-Hardware wird libx265 verwendet. Alte falsch
positive Fähigkeiten werden vor einem Batch neu geprüft. Zusätzliche AMD-,
Intel- und VA-API-Encoder/Kalibrierung des vollständigen Forks bleiben separat.
Ein kurzer synthetischer 8x-HEVC-Erzeugungs-/Decodiertest ist bestanden; das
ersetzt nicht die Abnahme echter Kamerajobs. Bei Bedarf kann nach dem kleinen
Wiedergabetest ein einzelner Front/8x-Job geprüft werden. In dieser separaten
Testinstanz muss FFmpeg einmal konfiguriert werden: typischer Pfad unter Ubuntu
/usr/bin/ffmpeg. Den Test-Knopf verwenden, bevor eine neue Berechnung startet.
Die CPU-Ausgabe kann langsamer als im vollständigen GPU-Fork sein.
