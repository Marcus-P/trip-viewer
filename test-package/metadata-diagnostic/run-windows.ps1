param(
    [Parameter(Mandatory = $true)][string]$TwoChannel,
    [Parameter(Mandatory = $true)][string]$ThreeChannel,
    [Parameter(Mandatory = $true)][string]$ToolDir,
    [string]$FFprobe,
    [string]$OutputDir
)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
if (-not [Environment]::Is64BitOperatingSystem -or $env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
    throw 'This launcher requires Windows x64.'
}
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
foreach ($tripInput in @($TwoChannel, $ThreeChannel)) {
    if (-not (Test-Path -LiteralPath $tripInput -PathType Container)) {
        throw 'One of the two input folders is missing. No recording was changed.'
    }
}
if (-not $FFprobe) {
    $tripProbeCommand = Get-Command ffprobe.exe -ErrorAction SilentlyContinue
    if ($tripProbeCommand) { $FFprobe = $tripProbeCommand.Source }
}
if (-not $FFprobe) {
    $tripCandidates = @(
        (Join-Path $env:ProgramFiles 'ffmpeg\bin\ffprobe.exe'),
        'C:\ffmpeg\bin\ffprobe.exe',
        (Join-Path $env:LOCALAPPDATA 'Microsoft\WinGet\Links\ffprobe.exe')
    )
    $tripFfmpegCommand = Get-Command ffmpeg.exe -ErrorAction SilentlyContinue
    if ($tripFfmpegCommand) {
        $tripCandidates += Join-Path (Split-Path $tripFfmpegCommand.Source) 'ffprobe.exe'
    }
    # Read only the existing configured executable path. Never print settings,
    # archive paths or unrelated application data, and never modify settings.
    foreach ($tripId in @('com.tripviewer.app', 'com.tripviewer.pr21test')) {
        $tripSettings = Join-Path $env:APPDATA ($tripId + '\settings.json')
        if (Test-Path -LiteralPath $tripSettings -PathType Leaf) {
            try {
                $tripConfigured = (Get-Content -Raw -LiteralPath $tripSettings | ConvertFrom-Json).ffmpeg_path
                if ($tripConfigured) {
                    $tripCandidates += Join-Path (Split-Path $tripConfigured) 'ffprobe.exe'
                }
            } catch { }
        }
    }
    $tripWinget = Join-Path $env:LOCALAPPDATA 'Microsoft\WinGet\Packages'
    if (Test-Path -LiteralPath $tripWinget -PathType Container) {
        $tripCandidates += @(Get-ChildItem -Path (Join-Path $tripWinget 'Gyan.FFmpeg*') `
            -Filter ffprobe.exe -Recurse -File -ErrorAction SilentlyContinue | Select-Object -ExpandProperty FullName)
    }
    foreach ($tripCandidate in $tripCandidates) {
        if (Test-Path -LiteralPath $tripCandidate -PathType Leaf) {
            $FFprobe = $tripCandidate
            break
        }
    }
}
if (-not $FFprobe -or -not (Test-Path -LiteralPath $FFprobe -PathType Leaf)) {
    Add-Type -AssemblyName System.Windows.Forms
    $tripPicker = New-Object System.Windows.Forms.OpenFileDialog
    $tripPicker.Title = 'Choose ffprobe.exe from your existing FFmpeg bin folder'
    $tripPicker.Filter = 'FFprobe|ffprobe.exe'
    if ($tripPicker.ShowDialog() -ne [System.Windows.Forms.DialogResult]::OK) {
        throw 'FFprobe was not selected. Nothing has been collected.'
    }
    $FFprobe = $tripPicker.FileName
}

$tripRuntimeSha = '0eb85c2dfccccf1b17352de4c397f69194035b7d37149eacc16f1147d93de3b8'
$tripRuntimeArchive = Join-Path $ToolDir 'python-runtime.zip'
$tripRuntimeDirectory = Join-Path $ToolDir 'python-runtime'
# Reuse only a checksum-verified archive from the earlier diagnostic. Extract
# fresh; do not trust an executable from a previous writable runtime directory.
$tripDownloadRoot = Split-Path $ToolDir -Parent
$tripArchives = @()
foreach ($tripPattern in @('TripViewer-Parkdiagnose*', 'TripViewer-Metadaten-*')) {
    $tripArchives += @(Get-ChildItem -Path (Join-Path $tripDownloadRoot ($tripPattern + '\python-runtime.zip')) `
        -File -ErrorAction SilentlyContinue)
}
foreach ($tripArchive in $tripArchives) {
    if ((Get-FileHash -Algorithm SHA256 -LiteralPath $tripArchive.FullName).Hash.ToLowerInvariant() -eq $tripRuntimeSha) {
        if ($tripArchive.FullName -ne $tripRuntimeArchive) {
            Copy-Item -LiteralPath $tripArchive.FullName -Destination $tripRuntimeArchive
        }
        break
    }
}
$tripRuntimeValid = (Test-Path -LiteralPath $tripRuntimeArchive -PathType Leaf) -and
    ((Get-FileHash -Algorithm SHA256 -LiteralPath $tripRuntimeArchive).Hash.ToLowerInvariant() -eq $tripRuntimeSha)
if (-not $tripRuntimeValid) {
    Write-Host 'Downloading portable Python (about 14 MB); no installation.'
    # The v2 endpoint redirects to globalcdn.nuget.org, which can fail DNS on
    # an otherwise working connection. Flat-container serves the SAME pinned
    # official package directly from api.nuget.org, without that redirect.
    foreach ($tripRuntimeUrl in @(
        'https://api.nuget.org/v3-flatcontainer/python/3.12.10/python.3.12.10.nupkg',
        'https://www.nuget.org/api/v2/package/python/3.12.10'
    )) {
        try {
            Invoke-WebRequest -UseBasicParsing -TimeoutSec 45 -Uri $tripRuntimeUrl -OutFile $tripRuntimeArchive
            $tripRuntimeValid = (Get-FileHash -Algorithm SHA256 -LiteralPath $tripRuntimeArchive).Hash.ToLowerInvariant() -eq $tripRuntimeSha
            if ($tripRuntimeValid) { break }
        } catch { }
    }
}
if (-not $tripRuntimeValid) {
    throw 'Neither official Python endpoint returned the checksum-verified package. No Python was executed.'
}
Expand-Archive -LiteralPath $tripRuntimeArchive -DestinationPath $tripRuntimeDirectory
$tripPython = Join-Path $tripRuntimeDirectory 'tools\python.exe'

$tripExifArchive = Join-Path $ToolDir 'exiftool-13.64_64.zip'
$tripExifDirectory = Join-Path $ToolDir 'exiftool-runtime'
Write-Host 'Downloading portable ExifTool 13.64 from its official SourceForge project (about 11 MB).'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$tripExifValid = $false
# Some SourceForge mirrors return an HTML page with HTTP 200. Follow the
# official download page's redirects and require an actual, readable ZIP.
$tripExifUrls = @(
    'https://sourceforge.net/projects/exiftool/files/exiftool-13.64_64.zip/download',
    'https://downloads.sourceforge.net/project/exiftool/exiftool-13.64_64.zip?use_mirror=phoenixnap'
)
foreach ($tripExifUrl in $tripExifUrls) {
    try {
        if (Get-Command curl.exe -ErrorAction SilentlyContinue) {
            & curl.exe --fail --location --silent --show-error --max-time 90 --output $tripExifArchive $tripExifUrl
            if ($LASTEXITCODE -ne 0) { continue }
        } else {
            Invoke-WebRequest -UseBasicParsing -Uri $tripExifUrl -OutFile $tripExifArchive
        }
        $tripZipCheck = [IO.Compression.ZipFile]::OpenRead($tripExifArchive)
        $tripZipCheck.Dispose()
        $tripExifValid = $true
        break
    } catch { }
}
if (-not $tripExifValid) {
    throw 'The official ExifTool download did not return a valid ZIP. No downloaded ExifTool was executed.'
}
$tripExifSha = '72c29c44d1e860eebcaefa71692d979af9858fe9a6d7d791a5e7f1dac77804af'
if ((Get-FileHash -Algorithm SHA256 -LiteralPath $tripExifArchive).Hash.ToLowerInvariant() -ne $tripExifSha) {
    throw 'ExifTool checksum mismatch. Nothing from that archive will be executed.'
}
Expand-Archive -LiteralPath $tripExifArchive -DestinationPath $tripExifDirectory
$tripExifOriginal = @(Get-ChildItem -LiteralPath $tripExifDirectory -Filter 'exiftool(-k).exe' -Recurse -File)
if ($tripExifOriginal.Count -ne 1) { throw 'Unexpected ExifTool package layout.' }
$tripExifTool = Join-Path $tripExifOriginal[0].DirectoryName 'exiftool.exe'
Rename-Item -LiteralPath $tripExifOriginal[0].FullName -NewName 'exiftool.exe'
Write-Host ('ExifTool archive SHA256: ' + (Get-FileHash -Algorithm SHA256 -LiteralPath $tripExifArchive).Hash.ToLowerInvariant())
if (-not $OutputDir) { $OutputDir = Join-Path $ToolDir 'metadata-results' }
$tripEntry = "import runpy,sys;sys.path.insert(0,sys.argv[1]);sys.argv=sys.argv[2:];runpy.run_path(sys.argv[0],run_name='__main__')"
Write-Host 'Reading metadata only. Raw results stay in PRIVATE-DO-NOT-SHARE. Nothing is uploaded.'
& $tripPython -I -c $tripEntry $ToolDir (Join-Path $ToolDir 'capture.py') `
    --set '2ch-lowbitrate' $TwoChannel --set '3ch-time2fps' $ThreeChannel `
    --exiftool $tripExifTool --ffprobe $FFprobe --output-dir $OutputDir
$tripCollectionExit = $LASTEXITCODE
$tripReportPath = Join-Path $OutputDir 'SHARE-metadata-report.json'
if (Test-Path -LiteralPath $tripReportPath -PathType Leaf) {
    Write-Host ('Report to review: ' + $tripReportPath)
    Write-Host 'Keep PRIVATE-DO-NOT-SHARE on your own computer. Do not upload that folder.'
}
if ($tripCollectionExit -ne 0) {
    throw 'Collection is partial or interrupted. Review the SHARE report; no original was changed.'
}
