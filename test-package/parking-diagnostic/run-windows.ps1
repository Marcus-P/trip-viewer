param(
    [Parameter(Mandatory = $true)][string]$TwoChannel,
    [Parameter(Mandatory = $true)][string]$ThreeChannel,
    [Parameter(Mandatory = $true)][string]$ToolDir,
    [string]$ReportPath
)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
if (-not [Environment]::Is64BitOperatingSystem -or $env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
    throw 'This portable launcher requires Windows x64.'
}
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$tripRuntimeArchive = Join-Path $ToolDir 'python-runtime.zip'
$tripRuntimeDirectory = Join-Path $ToolDir 'python-runtime'
$tripRuntimeUrl = 'https://www.nuget.org/api/v2/package/python/3.12.10'
$tripRuntimeSha = '0eb85c2dfccccf1b17352de4c397f69194035b7d37149eacc16f1147d93de3b8'
Write-Host 'Downloading the portable Python Software Foundation runtime (about 14 MB)...'
Invoke-WebRequest -UseBasicParsing -Uri $tripRuntimeUrl -OutFile $tripRuntimeArchive
$tripActualSha = (Get-FileHash -Algorithm SHA256 -LiteralPath $tripRuntimeArchive).Hash.ToLowerInvariant()
if ($tripActualSha -ne $tripRuntimeSha) {
    throw 'Python runtime checksum mismatch. Nothing will be executed.'
}
Expand-Archive -LiteralPath $tripRuntimeArchive -DestinationPath $tripRuntimeDirectory
$tripPython = Join-Path $tripRuntimeDirectory 'tools\python.exe'
if (-not (Test-Path -LiteralPath $tripPython -PathType Leaf)) {
    throw 'The portable Python runtime is incomplete.'
}
if (-not $ReportPath) {
    $ReportPath = Join-Path $ToolDir 'viofo-parking-report.json'
}
$tripEntry = "import runpy,sys;sys.path.insert(0,sys.argv[1]);sys.argv=sys.argv[2:];runpy.run_path(sys.argv[0],run_name='__main__')"
Write-Host 'Reading the two recording sets. No files will be uploaded.'
& $tripPython -I -c $tripEntry $ToolDir (Join-Path $ToolDir 'diagnose.py') `
    --set '2ch-lowbitrate' $TwoChannel `
    --set '3ch-time2fps' $ThreeChannel `
    --output $ReportPath
if ($LASTEXITCODE -ne 0) {
    throw 'The diagnostic stopped. A partial report may be available; no original was changed.'
}
if (-not (Test-Path -LiteralPath $ReportPath -PathType Leaf)) {
    throw 'No diagnostic report was created.'
}
$tripReport = Get-Content -Raw -LiteralPath $ReportPath | ConvertFrom-Json
Write-Host ('Collection status: ' + $tripReport.status)
Write-Host ('Report: ' + $ReportPath)
Write-Host 'Share only the reviewed JSON report, not recordings or terminal logs.'
