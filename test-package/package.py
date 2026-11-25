from pathlib import Path
import hashlib, json, os, shutil, subprocess, tarfile
root=Path.cwd()
name="TripViewer-PR21-Test-17f6f97"
out=root/"test-package/out"; out.mkdir()
p=out/name;p.mkdir()
# The prior published candidate omitted this plugin. File presence plus
# factory loading is checked before publishing, without requiring a CI GPU.
appdirs=list((root/"src-tauri/target/release/bundle/appimage").glob("*.AppDir"))
assert len(appdirs)==1, appdirs
appdir=appdirs[0]
assert (appdir/"usr/lib/gstreamer-1.0/libgstopengl.so").is_file(), "GL media plugin missing from AppImage source"
media_env=os.environ.copy()
media_env.update(GST_PLUGIN_SYSTEM_PATH_1_0=str(appdir/"usr/lib/gstreamer-1.0"),
 GST_PLUGIN_PATH_1_0=str(appdir/"usr/lib/gstreamer-1.0"),
 GST_PLUGIN_SCANNER_1_0=str(appdir/"usr/lib/gstreamer1.0/gstreamer-1.0/gst-plugin-scanner"),
 GST_REGISTRY=str(out/"gst-registry-check.bin"),
 LD_LIBRARY_PATH=str(appdir/"usr/lib")+":"+os.environ.get("LD_LIBRARY_PATH",""))
for factory in ["glupload","glcolorconvert","gldownload","appsink"]:
 check=subprocess.run(["gst-inspect-1.0",factory],env=media_env,capture_output=True,text=True,timeout=30)
 assert check.returncode==0, "Bundled GStreamer factory unavailable: "+factory+"\n"+check.stderr
images=list((root/"src-tauri/target/release/bundle/appimage").glob("*.AppImage"))
assert len(images)==1, images
shutil.copyfile(images[0],p/"TripViewer-PR21-Test.AppImage")
for f in ["start-test.sh","README-DE.txt","THIRD_PARTY_NOTICES.md"]:
 shutil.copyfile(root/"test-package"/f,p/f)
shutil.copyfile(root/"LICENSE",p/"LICENSE")
for f in ["start-test.sh","TripViewer-PR21-Test.AppImage"]:(p/f).chmod(0o755)
info={"source_commit":"17f6f97b1bb1c35a82a885fdc1bfa13c7dca404c","base_snapshot":"462eeaa49e23519aeea1fde2ac1031d9848c3e6b","packaging_commit":os.environ["GITHUB_SHA"],"runner":"ubuntu-24.04","application_identifier":"com.tripviewer.pr21test","updater_endpoints":[],"owner_media_playback":"pending","upstream_pr_candidate":"f7bb990be22ae117fe99cfb263b29294d225c070","bundled_gl_factories_checked":True}
(p/"BUILD-INFO.json").write_text(json.dumps(info,indent=2)+"\n")
# Retain available native redistribution notices from the packaging host.
for f in Path("/usr/share/doc").glob("*/copyright"):
 if f.is_file():
  dest=p/"native-license-notices"/f.parent.name/"copyright";dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(f,dest)
env=os.environ.copy();scratch=out/"startup-check"
# Only the virtual CI display uses software compositing. The shipped launcher
# preserves the real computer's normal graphics/media configuration.
env.update(TRIPVIEWER_PR21_HOME=str(scratch/"settings"),TRIPVIEWER_PR21_ARCHIVE=str(scratch/"archive"),APPIMAGE_EXTRACT_AND_RUN="1",
 LIBGL_ALWAYS_SOFTWARE="1",WEBKIT_DISABLE_COMPOSITING_MODE="1")
import signal,threading
from types import SimpleNamespace
process=subprocess.Popen(["xvfb-run","-a","dbus-run-session","--",str(p/"start-test.sh")],
 env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,start_new_session=True)
output=[]; changed=threading.Event()
def collect_output():
 for line in process.stdout:
  output.append(line)
  if "scan_folder: found 0 video files" in line: changed.set()
 changed.set()  # Unexpected exit is also terminal; do not wait out the bound.
reader=threading.Thread(target=collect_output,daemon=True);reader.start()
try:
 signalled=changed.wait(90)  # Failure deadline, never evidence of success.
 live=process.poll() is None
finally:
 try: os.killpg(process.pid,signal.SIGTERM)
 except ProcessLookupError: pass
 try: process.wait(timeout=10)
 except subprocess.TimeoutExpired:
  os.killpg(process.pid,signal.SIGKILL);process.wait()
 reader.join(timeout=2)
check=SimpleNamespace(stdout="".join(output),stderr="",returncode=process.returncode)
logpath=scratch/"settings/last-run.log"
log=logpath.read_text() if logpath.exists() else "<no app log>"
def startup_check(condition, reason):
 if condition:return
 detail=(reason+"\n"+check.stdout+"\n"+log)[-12000:]
 escaped=detail.replace("%","%25").replace("\r","%0D").replace("\n","%0A")
 print("::error title=Isolated package startup::"+escaped, flush=True)
 raise RuntimeError(reason)
startup_check(signalled and live and "scan_folder: found 0 video files" in check.stdout,
 "Packaged frontend did not reach its scan event before exit/deadline")
settings=scratch/"settings/data/com.tripviewer.pr21test/settings.json"
assert json.loads(settings.read_text())["last_archive"]==str(scratch/"archive")
dbfile=scratch/"archive/.tripviewer/tripviewer.db"
assert dbfile.exists()
import sqlite3
with sqlite3.connect(dbfile) as db:
 assert db.execute("PRAGMA user_version").fetchone()[0]==13
info["archive_schema_version"]=13
info["analysis_storage"]="separate disposable analysis-cache-v1.sqlite"
log=(scratch/"settings/last-run.log").read_text()
startup_check("scan_folder: found 0 video files" in check.stdout, "Packaged frontend did not scan the empty scratch archive")
startup_check("Unable to spawn" not in log and "appsink not found" not in log, "Packaged media runtime dependency failure")
info["isolated_packaged_startup"]="passed on Ubuntu 24.04 virtual display with software compositing; no owner media loaded"
(p/"BUILD-INFO.json").write_text(json.dumps(info,indent=2)+"\n")
files=sorted(f for f in p.rglob("*") if f.is_file())
(p/"SHA256SUMS.txt").write_text("".join(hashlib.file_digest(f.open("rb"),"sha256").hexdigest()+"  "+str(f.relative_to(p))+"\n" for f in files))
archive=out/(name+"-Ubuntu-x86_64.tar.gz")
with tarfile.open(archive,"w:gz",compresslevel=1) as t:
 t.add(p,arcname=name)
(out/"TEST-PACKAGE-SHA256SUMS.txt").write_text(hashlib.file_digest(archive.open("rb"),"sha256").hexdigest()+"  "+archive.name+"\n")
print("Packaged and startup-checked:",archive.name)
