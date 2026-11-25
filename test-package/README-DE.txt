Isolierter PR21-Wiedergabetest für Ubuntu 24.04 x86_64

Vor dem Start die alte PR21-Testversion schließen. Die vorhandenen Videokopien
und manuellen Zusammenführungen im bisherigen Testarchiv werden weiterverwendet:
  ~/Downloads/TripViewer-PR21-Testarchiv-5dbdaf9
Kein produktives Archiv öffnen. Die installierte 0.13.0 bleibt unverändert.

Nach dem Entpacken im Paketordner:
  sha256sum -c SHA256SUMS.txt
  TRIPVIEWER_GPS_TIMING=1 bash ./start-test.sh

Fenstertitel: Trip Viewer PR21 Test — a61c7e7
Testversion: 0.3.4-rc.2026112506
Separate Einstellungen und Log:
  ~/.local/share/tripviewer-pr21-test/a61c7e7/last-run.log
Die Ausgaben erscheinen auch direkt im Terminal.

Die Hauptdatenbank bleibt bei Schema13. Analyse- und Medienfehlerdaten stehen
separat in .tripviewer/analysis-cache-v1.sqlite. GPS-Reader-Version9 bleibt
unverändert; gültige bisherige Ergebnisse können wiederverwendet werden.
Keine neue Timelapse-Codierung und kein Kopieren oder Löschen nötig.

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
