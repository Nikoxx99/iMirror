param(
    [string]$DriverPath = $PSScriptRoot,
    [string]$LogPath = ""
)

$ErrorActionPreference = "Stop"
$script:TranscriptStarted = $false

if ($LogPath) {
    try {
        Start-Transcript -Path $LogPath -Force | Out-Null
        $script:TranscriptStarted = $true
    } catch {
        Write-Warning "Could not start installer diagnostics: $($_.Exception.Message)"
    }
}

trap {
    Write-Host "ERROR: $($_ | Out-String)" -ForegroundColor Red
    if ($script:TranscriptStarted) {
        Stop-Transcript | Out-Null
    }
    exit 1
}

function Assert-Administrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw "Run this script from an Administrator PowerShell window."
    }
}

function Convert-ToNativeWindowsPath {
    param([string]$Path)

    $native = $Path
    $providerPrefix = "Microsoft.PowerShell.Core\FileSystem::"
    if ($native.StartsWith($providerPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        $native = $native.Substring($providerPrefix.Length)
    }
    if ($native.StartsWith('\\?\UNC\', [StringComparison]::OrdinalIgnoreCase)) {
        return "\\$($native.Substring(8))"
    }
    if ($native.StartsWith('\\?\', [StringComparison]::OrdinalIgnoreCase)) {
        return $native.Substring(4)
    }
    return [IO.Path]::GetFullPath($native)
}

function Unregister-Filter {
    param(
        [string]$RegSvr32,
        [string]$DllPath
    )

    if (Test-Path -LiteralPath $DllPath) {
        Unblock-DriverDll -DllPath $DllPath
        $process = Start-Process -FilePath $RegSvr32 -ArgumentList @("/s", "/u", "`"$DllPath`"") -Wait -PassThru -WindowStyle Hidden
        if ($process.ExitCode -ne 0) {
            throw "regsvr32 unregister failed for $DllPath with exit code $($process.ExitCode)"
        }
    }
}

function Unblock-DriverDll {
    param([string]$DllPath)

    try {
        $zone = Get-Item -LiteralPath $DllPath -Stream Zone.Identifier -ErrorAction SilentlyContinue
        if ($zone) {
            Write-Host "Removing downloaded-file block from $([IO.Path]::GetFileName($DllPath))..."
            Unblock-File -LiteralPath $DllPath
        }
    } catch {
        Write-Warning "Could not remove downloaded-file block from $DllPath. $($_.Exception.Message)"
    }
}

function Send-DeviceChangeBroadcast {
    Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

public static class IMirrorDeviceBroadcast {
    [DllImport("user32.dll", SetLastError = true)]
    public static extern IntPtr SendMessageTimeout(
        IntPtr hWnd,
        uint Msg,
        IntPtr wParam,
        IntPtr lParam,
        uint fuFlags,
        uint uTimeout,
        out IntPtr lpdwResult);
}
"@ -ErrorAction SilentlyContinue

    $result = [IntPtr]::Zero
    [IMirrorDeviceBroadcast]::SendMessageTimeout([IntPtr]0xffff, 0x0219, [IntPtr]::Zero, [IntPtr]::Zero, 2, 1000, [ref]$result) | Out-Null
}

Write-Host "iMirror Camera driver removal" -ForegroundColor Cyan
Assert-Administrator

$resolvedDriverPath = Convert-ToNativeWindowsPath -Path $DriverPath
$dll64 = Join-Path $resolvedDriverPath "UnityCaptureFilter64.dll"

Write-Host "Using native driver path: $resolvedDriverPath"
Write-Host "Unregistering 64-bit DirectShow filter..."
Unregister-Filter -RegSvr32 "$env:SystemRoot\System32\regsvr32.exe" -DllPath $dll64

$installInfo = Join-Path $env:APPDATA "iMirror\driver-install.json"
if (Test-Path -LiteralPath $installInfo) {
    Remove-Item -LiteralPath $installInfo -Force
}

Send-DeviceChangeBroadcast

Write-Host ""
Write-Host "Removal complete." -ForegroundColor Green
Write-Host "Restart Chrome or any app that had the camera picker open."

if ($script:TranscriptStarted) {
    Stop-Transcript | Out-Null
}
exit 0
