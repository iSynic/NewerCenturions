param([string]$GhidraRoot = 'F:\Ghidra\ghidra_12.1.2_PUBLIC')
$ErrorActionPreference = 'Stop'
$moduleRoot = Join-Path $env:APPDATA 'ghidra\ghidra_12.1.2_PUBLIC\Extensions\Centurions68k'
$languageRoot = Join-Path $moduleRoot 'data\languages'
New-Item -ItemType Directory -Force -Path $languageRoot | Out-Null
$source = Join-Path $GhidraRoot 'Ghidra\Processors\68000\data\languages'
foreach ($name in @('68000.sinc', '68000.pspec', '68000.cspec')) {
    Copy-Item -LiteralPath (Join-Path $source $name) -Destination $languageRoot
}
$compiler = Join-Path $languageRoot '68000.cspec'
$spec = Get-Content -LiteralPath $compiler -Raw
$spec = $spec.Replace('<integer_size value="4" />', '<integer_size value="2" />').Replace('maxsize="500" align="4"', 'maxsize="500" align="2"')
$spec = $spec.Replace('<output>', '<output><pentry minsize="4" maxsize="4" metatype="ptr"><register name="A0" /></pentry>')
Set-Content -LiteralPath $compiler -Value $spec -Encoding utf8
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'ghidra\mac68k\Centurions.slaspec') -Destination $languageRoot
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'ghidra\mac68k\Centurions.ldefs') -Destination $languageRoot
Set-Content -LiteralPath (Join-Path $moduleRoot 'Module.manifest') -Value ''
Set-Content -LiteralPath (Join-Path $moduleRoot 'extension.properties') -Value @('name=Centurions68k', 'description=68k Classic Mac trap instruction boundaries', 'author=', 'createdOn=2026-10-09', 'version=12.1.2')
& (Join-Path $GhidraRoot 'support\sleigh.bat') (Join-Path $languageRoot 'Centurions.slaspec')
if ($LASTEXITCODE) { throw 'Sleigh compilation failed' }
Write-Output "Installed local language in $moduleRoot"
