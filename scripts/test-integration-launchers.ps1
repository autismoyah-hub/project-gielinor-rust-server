$ErrorActionPreference='Stop'
function Read-ScriptAst([string]$name) {
    $parseTokens=$null; $parseErrors=$null
    $ast=[System.Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot $name),[ref]$parseTokens,[ref]$parseErrors)
    if ($parseErrors.Count) { throw "$name has parse errors: $parseErrors" }
    return $ast
}
foreach ($name in @('integration-kotlin-login.ps1','integration-rust-login.ps1')) {
    $ast=Read-ScriptAst $name
    $selector=$ast.FindAll({param($node) $node -is [System.Management.Automation.Language.AssignmentStatementAst] -and $node.Left.Extent.Text -eq '$binary'},$true)
    if ($selector.Count -ne 1) { throw 'Expected one binary selector' }
    $ClientRoot='C:/owned/pinned'; $scenarioBinary='C:/owned/driver/server_tower.exe'
    foreach ($Mode in @('tower-diary','login','render','actions','prayer','arrival','dialogue-ui','dialogue-protocol','inventory','inventory-verify','inventory-bury','floor','floor-content')) {
        $selected=Invoke-Expression $selector[0].Right.Extent.Text
        $expected=switch ($Mode) { 'login' {"$ClientRoot/bin/live_session.exe"} 'render' {"$ClientRoot/bin/gielinor-client.exe"} 'actions' {"$ClientRoot/bin/live_actions.exe"} default {$scenarioBinary} }
        if ($selected -ne $expected) { throw "Wrong executable identity for $name / $Mode" }
    }
    Write-Output "${name}: 13 executable identities PASS"
}
$ast=Read-ScriptAst 'integration-preparation-run.ps1'
$init=$ast.FindAll({param($node) $node -is [System.Management.Automation.Language.AssignmentStatementAst] -and $node.Left.Extent.Text.EndsWith('$diaryArgs')},$true)[0]
$branch=$ast.FindAll({param($node) $node -is [System.Management.Automation.Language.IfStatementAst] -and $node.Clauses[0].Item1.Extent.Text -eq '$KnownEmptyDiary' -and $node.Extent.Text -like '*diaryArgs*'},$true)
if (-not $init -or $branch.Count -ne 1) { throw 'Cannot locate fixture argument preparation' }
$python='C:/Users/backup/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe'
foreach ($KnownEmptyDiary in @($false,$true)) {
    Invoke-Expression $init.Extent.Text
    Invoke-Expression $branch[0].Extent.Text
    $actual=& $python -c 'import json,sys; print(json.dumps(sys.argv[1:]))' @diaryArgs
    if ($LASTEXITCODE -ne 0) { throw 'Native argv probe failed' }
    $expected=if ($KnownEmptyDiary) {'["--known-empty-diary"]'} else {'[]'}
    if ($actual -ne $expected) { throw "Fixture argv mismatch: $actual" }
}
Write-Output 'KnownEmptyDiary false/true native argument vectors PASS'
