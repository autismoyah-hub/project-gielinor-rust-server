param(
    [Parameter(Mandatory=$true)][string]$CacheRoot,
    [Parameter(Mandatory=$true)][string]$OutputRoot,
    [Parameter(Mandatory=$true)][string]$Blender,
    [Parameter(Mandatory=$true)][string]$Renderer,
    [Parameter(Mandatory=$true)][string]$IndependentAuthoringScript,
    [string]$Python='python'
)
$ErrorActionPreference='Stop'
$sourceTools=(Resolve-Path "$PSScriptRoot/..").Path
$serverRoot=(Resolve-Path "$PSScriptRoot/../../../..").Path
$cachePath=(Resolve-Path $CacheRoot).Path
$outputPath=[IO.Path]::GetFullPath($OutputRoot)
if($outputPath.StartsWith($serverRoot,[StringComparison]::OrdinalIgnoreCase) -or $outputPath.StartsWith($cachePath,[StringComparison]::OrdinalIgnoreCase)) {throw 'Protected derivatives must be outside the repository and cache.'}
New-Item -ItemType Directory -Force -Path $outputPath | Out-Null
$finalRoot=Join-Path $outputPath 'final'
$reference=Join-Path $outputPath 'reference-final'
function Confirm-Exit {if($LASTEXITCODE -ne 0){throw "Calibration operation failed ($LASTEXITCODE)"}}
& $Renderer $cachePath $reference export;Confirm-Exit
& $Blender -b --factory-startup --python "$sourceTools/blender/calibration_roundtrip.py" -- --source $reference --output "$finalRoot/blender-roundtrip";Confirm-Exit
& $Blender -b "$finalRoot/blender-roundtrip/roundtrip.blend" --python "$sourceTools/blender/calibration_reopen.py" -- "$finalRoot/reopened-roundtrip";Confirm-Exit
& $Blender -b "$finalRoot/blender-roundtrip/roundtrip.blend" --python "$sourceTools/blender/calibration_reopen.py" -- "$finalRoot/mutation-witness" --witness;Confirm-Exit
& $Blender -b --factory-startup --python $IndependentAuthoringScript -- "$finalRoot/independent-authored" $reference "$sourceTools/blender";Confirm-Exit
& $Python "$sourceTools/converter/calibration_codec.py" "$finalRoot/reopened-roundtrip" "$finalRoot/roundtrip-package" $reference;Confirm-Exit
& $Python "$sourceTools/converter/calibration_codec.py" "$finalRoot/independent-authored" "$finalRoot/independent-package" $reference;Confirm-Exit
foreach($kind in @('static','npc')) {
    & $Renderer $cachePath "$finalRoot/native-$kind" render $kind original original;Confirm-Exit
    foreach($prefix in @('roundtrip','independent')) {
        & $Renderer $cachePath "$finalRoot/$prefix-$kind" render $kind "$finalRoot/$prefix-package/$kind.model" "$finalRoot/$prefix-package";Confirm-Exit
    }
    & $Renderer $cachePath "$finalRoot/compare-$kind" compare $kind original "$finalRoot/roundtrip-package/$kind.model" "$finalRoot/independent-package/$kind.model";Confirm-Exit
}
& $Python "$sourceTools/design/compare_asset_calibration.py" $finalRoot;Confirm-Exit
& $Python "$sourceTools/design/review_asset_calibration.py" $finalRoot;Confirm-Exit
$identity=@{server_base_commit=(& git -C $serverRoot rev-parse HEAD);server_worktree_status=(& git -C $serverRoot status --porcelain);client=(& git -C "$serverRoot/../rust-client-reference" rev-parse HEAD);renderer_sha256=(Get-FileHash -LiteralPath $Renderer -Algorithm SHA256).Hash;native_source_sha256=(Get-FileHash -LiteralPath "$serverRoot/tools/tower/offline/render/src/bin/asset_calibration.rs" -Algorithm SHA256).Hash;private_recipe_sha256=(Get-FileHash -LiteralPath $IndependentAuthoringScript -Algorithm SHA256).Hash;cache_access='read only';production_integration=$false}
$identity|ConvertTo-Json|Set-Content -LiteralPath "$finalRoot/source-identities.json"
Write-Output "Private review: $finalRoot/review/index.html"
