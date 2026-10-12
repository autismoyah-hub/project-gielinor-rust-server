$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
$referenceRoot = 'C:\Projects\OSRSRSPS\Project-Gielinor'
$revision = 'cb6e66cc0f16a25b60ba6705d8f157cc22ab4869'
if (Test-Path "$serverRoot/reference/kotlin") { throw 'Snapshot already exists; preserve it rather than overwriting.' }
New-Item -ItemType Directory -Force -Path "$serverRoot/work", "$serverRoot/reference" | Out-Null
& git -c safe.directory=C:/Projects/OSRSRSPS/Project-Gielinor -C $referenceRoot archive --format=zip "--output=$serverRoot/work/kotlin-baseline.zip" $revision api engine content server build-logic gradle build.gradle.kts settings.gradle.kts gradle.properties gradlew gradlew.bat LICENSE.md README.md
if ($LASTEXITCODE -ne 0) { throw 'Reference archive failed.' }
Expand-Archive -LiteralPath "$serverRoot/work/kotlin-baseline.zip" -DestinationPath "$serverRoot/reference/kotlin"
& git -c safe.directory=C:/Projects/OSRSRSPS/Project-Gielinor -C $referenceRoot status --short | Set-Content "$serverRoot/work/reference-local-changes.txt"
