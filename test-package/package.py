from pathlib import Path
import hashlib, json, os, shutil, subprocess, tarfile
root=Path.cwd()
name="TripViewer-PR21-Test-680a9b2"
out=root/"test-package/out"; out.mkdir()
p=out/name;p.mkdir()
images=list((root/"src-tauri/target/release/bundle/appimage").glob("*.AppImage"))
assert len(images)==1, images
shutil.copyfile(images[0],p/"TripViewer-PR21-Test.AppImage")
for f in ["start-test.sh","README-DE.txt","THIRD_PARTY_NOTICES.md"]:
 shutil.copyfile(root/"test-package"/f,p/f)
shutil.copyfile(root/"LICENSE",p/"LICENSE")
for f in ["start-test.sh","TripViewer-PR21-Test.AppImage"]:(p/f).chmod(0o755)
info={"source_commit":"680a9b2ec3838daddb55cb865d83b09fc917f277","base_snapshot":"462eeaa49e23519aeea1fde2ac1031d9848c3e6b","packaging_commit":os.environ["GITHUB_SHA"],"runner":"ubuntu-24.04","application_identifier":"com.tripviewer.pr21test","updater_endpoints":[],"owner_media_playback":"pending"}
(p/"BUILD-INFO.json").write_text(json.dumps(info,indent=2)+"\n")
# Retain available native redistribution notices from the packaging host.
for f in Path("/usr/share/doc").glob("*/copyright"):
 if f.is_file():
  dest=p/"native-license-notices"/f.parent.name/"copyright";dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(f,dest)
env=os.environ.copy();scratch=out/"startup-check"
env.update(TRIPVIEWER_PR21_HOME=str(scratch/"settings"),TRIPVIEWER_PR21_ARCHIVE=str(scratch/"archive"),APPIMAGE_EXTRACT_AND_RUN="1")
# Timeout only bounds this smoke check; grouping uses no runtime waits.
check=subprocess.run(["timeout","--signal=TERM","12s","xvfb-run","-a","dbus-run-session","--",str(p/"start-test.sh")],env=env,capture_output=True,text=True)
logpath=scratch/"settings/last-run.log"
log=logpath.read_text() if logpath.exists() else "<no app log>"
def startup_check(condition, reason):
 if condition:return
 detail=(reason+"\n"+check.stdout+"\n"+check.stderr+"\n"+log)[-12000:]
 escaped=detail.replace("%","%25").replace("\r","%0D").replace("\n","%0A")
 print("::error title=Isolated package startup::"+escaped, flush=True)
 raise RuntimeError(reason)
startup_check(check.returncode==124, "Unexpected packaged application exit: "+str(check.returncode))
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
startup_check("scan_folder: found 0 video files" in log, "Packaged frontend did not scan the empty scratch archive")
startup_check("Unable to spawn" not in log and "appsink not found" not in log, "Packaged media runtime dependency failure")
info["isolated_packaged_startup"]="passed on Ubuntu 24.04; no owner media loaded"
(p/"BUILD-INFO.json").write_text(json.dumps(info,indent=2)+"\n")
files=sorted(f for f in p.rglob("*") if f.is_file())
(p/"SHA256SUMS.txt").write_text("".join(hashlib.file_digest(f.open("rb"),"sha256").hexdigest()+"  "+str(f.relative_to(p))+"\n" for f in files))
archive=out/(name+"-Ubuntu-x86_64.tar.gz")
with tarfile.open(archive,"w:gz",compresslevel=1) as t:
 t.add(p,arcname=name)
(out/"TEST-PACKAGE-SHA256SUMS.txt").write_text(hashlib.file_digest(archive.open("rb"),"sha256").hexdigest()+"  "+archive.name+"\n")
print("Packaged and startup-checked:",archive.name)
