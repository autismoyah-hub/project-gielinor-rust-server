$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
$generated = [Collections.Generic.List[string]]::new()
$generated.Add('// Generated from pinned RSProt 1.0.0-ALPHA-20260630. See reference/protocol.')
$generated.Add('use crate::framing::{PacketDef, Size};')
foreach ($entry in @(@('CLIENT','client'),@('SERVER','server'),@('LOGIN_CLIENT','login-client'),@('LOGIN_SERVER','login-server'))) {
    $generated.Add("pub static $($entry[0]): &[PacketDef] = &[")
    $unassigned = [Collections.Generic.List[string]]::new()
    foreach ($line in Get-Content "$serverRoot/reference/protocol/$($entry[1]).txt") {
        $parts = $line.Split(' ')
        if ([int]$parts[1] -lt 0) { $unassigned.Add($parts[0]); continue }
        $size = switch ([int]$parts[2]) { -1 { 'Size::U8' } -2 { 'Size::U16' } default { "Size::Fixed($_)" } }
        $generated.Add(('    PacketDef {{ opcode: {0}, size: {1}, name: "{2}" }},' -f $parts[1],$size,$parts[0]))
    }
    $generated.Add('];')
    $generated.Add("// Reference entries with no standalone wire opcode; not silently mapped to 255.")
    $generated.Add("pub static $($entry[0])_UNASSIGNED: &[&str] = &[")
    foreach ($name in $unassigned) { $generated.Add(('    "{0}",' -f $name)) }
    $generated.Add('];')
}
$generated | Set-Content "$serverRoot/crates/protocol/src/tables.rs"
