# snippet from https://serverfault.com/a/1058407/71017
if (!
    (New-Object Security.Principal.WindowsPrincipal(
        [Security.Principal.WindowsIdentity]::GetCurrent()
    )).IsInRole(
        [Security.Principal.WindowsBuiltInRole]::Administrator
    )
) {
    Start-Process `
        -FilePath 'powershell' `
        -ArgumentList (
        '-File', $MyInvocation.MyCommand.Source, $args `
        | % { $_ }
    ) `
        -Verb RunAs
    exit
}

$WshShell = New-Object -comObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("$Env:USERPROFILE\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup\GameTimeManager.lnk")
$Shortcut.TargetPath = "$PSScriptRoot\GameTimeManager.exe"
$Shortcut.Save()

Invoke-WmiMethod -Class Win32_Process -Name Create -ArgumentList "$PSScriptRoot\GameTimeManager.exe"

Write-Output "Successfully installed and started Game Time Monitor."

Pause