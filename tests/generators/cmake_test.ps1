#!/usr/bin/env pwsh
# Test rninja with CMake-generated Ninja files

$ErrorActionPreference = "Stop"

function Write-Status {
    param(
        [string]$Message,
        [ConsoleColor]$Color = [ConsoleColor]::White
    )

    Write-Host $Message -ForegroundColor $Color
}

function Join-CommandArguments {
    param([string[]]$Arguments = @())

    ($Arguments | ForEach-Object {
        if ($_ -notmatch '[\s"]') {
            $_
        }
        else {
            '"' + ($_ -replace '"', '\"') + '"'
        }
    }) -join " "
}

function Invoke-Quiet {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FilePath,

        [string[]]$Arguments = @(),

        [hashtable]$Environment = @{}
    )

    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo.FileName = $FilePath
    $process.StartInfo.Arguments = Join-CommandArguments -Arguments $Arguments
    $process.StartInfo.UseShellExecute = $false
    $process.StartInfo.RedirectStandardOutput = $true
    $process.StartInfo.RedirectStandardError = $true

    foreach ($name in $Environment.Keys) {
        $process.StartInfo.EnvironmentVariables[$name] = [string]$Environment[$name]
    }

    [void]$process.Start()
    $stdoutTask = $process.StandardOutput.ReadToEndAsync()
    $stderrTask = $process.StandardError.ReadToEndAsync()
    $process.WaitForExit()
    $process.WaitForExit()
    $stdoutTask.Wait()
    $stderrTask.Wait()
    return $process.ExitCode
}

function Invoke-QuietWithTimeout {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FilePath,

        [string[]]$Arguments = @(),

        [hashtable]$Environment = @{},

        [int]$TimeoutSeconds = 30
    )

    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo.FileName = $FilePath
    $process.StartInfo.Arguments = Join-CommandArguments -Arguments $Arguments
    $process.StartInfo.UseShellExecute = $false
    $process.StartInfo.RedirectStandardOutput = $true
    $process.StartInfo.RedirectStandardError = $true

    foreach ($name in $Environment.Keys) {
        $process.StartInfo.EnvironmentVariables[$name] = [string]$Environment[$name]
    }

    [void]$process.Start()
    $stdoutTask = $process.StandardOutput.ReadToEndAsync()
    $stderrTask = $process.StandardError.ReadToEndAsync()

    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        $process.Kill()
        return 124
    }

    $process.WaitForExit()
    $stdoutTask.Wait()
    $stderrTask.Wait()
    return $process.ExitCode
}

$scriptDir = $PSScriptRoot
$projectRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)

$isWindowsPlatform = $env:OS -eq "Windows_NT"
$rninjaName = if ($isWindowsPlatform) { "rninja.exe" } else { "rninja" }
$rninja = Join-Path $projectRoot "target/release/$rninjaName"
$ninja = "ninja"

Write-Status "=== CMake Generator Compatibility Test ===" Blue
Write-Host ""

if (-not (Get-Command cmake -ErrorAction SilentlyContinue)) {
    Write-Status "SKIP: cmake not found" Yellow
    exit 0
}

if (-not (Get-Command $ninja -ErrorAction SilentlyContinue)) {
    Write-Status "SKIP: ninja not found" Yellow
    exit 0
}

if (-not (Test-Path -LiteralPath $rninja -PathType Leaf)) {
    $cargoResult = Invoke-Quiet -FilePath "cargo" -Arguments @("build", "--release", "--manifest-path", (Join-Path $projectRoot "Cargo.toml"))
    if ($cargoResult -ne 0) {
        Write-Status "FAIL: cargo build failed with exit code $cargoResult" Red
        exit $cargoResult
    }
}

$testDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Path $testDir | Out-Null

try {
    Write-Host "Creating CMake project in $testDir"

    @"
cmake_minimum_required(VERSION 3.10)
project(test_project C)

add_library(mylib STATIC src/lib.c)
add_executable(myapp src/main.c)
target_link_libraries(myapp mylib)
"@ | Set-Content -LiteralPath (Join-Path $testDir "CMakeLists.txt") -NoNewline -Encoding ascii

    $srcDir = Join-Path $testDir "src"
    New-Item -ItemType Directory -Path $srcDir | Out-Null

    "int add(int a, int b) { return a + b; }" | Set-Content -LiteralPath (Join-Path $srcDir "lib.c") -NoNewline -Encoding ascii

    @"
#include <stdio.h>
extern int add(int a, int b);
int main() { printf("%d\n", add(1, 2)); return 0; }
"@ | Set-Content -LiteralPath (Join-Path $srcDir "main.c") -NoNewline -Encoding ascii

    $buildDir = Join-Path $testDir "build"
    New-Item -ItemType Directory -Path $buildDir | Out-Null
    Push-Location $buildDir

    try {
        Write-Status "Running cmake -G Ninja..." Yellow
        $cmakeResult = Invoke-Quiet -FilePath "cmake" -Arguments @("-G", "Ninja", "..")
        if ($cmakeResult -ne 0) {
            Write-Status "FAIL: cmake failed with exit code $cmakeResult" Red
            exit $cmakeResult
        }

        if (-not (Test-Path -LiteralPath "build.ninja" -PathType Leaf)) {
            Write-Status "FAIL: CMake did not generate build.ninja" Red
            exit 1
        }

        $buildStatementCount = (Select-String -LiteralPath "build.ninja" -Pattern "^build " | Measure-Object).Count
        Write-Host "Generated build.ninja with $buildStatementCount build statements"

        Write-Status "Building with Ninja..." Yellow
        [void](Invoke-Quiet -FilePath $ninja -Arguments @("clean"))
        $ninjaResult = Invoke-Quiet -FilePath $ninja

        Write-Status "Building with rninja..." Yellow
        [void](Invoke-Quiet -FilePath $ninja -Arguments @("clean"))
        $rninjaResult = Invoke-Quiet -FilePath $rninja -Arguments @("--no-daemon") -Environment @{ RNINJA_CACHE_ENABLED = "0" }

        if ($ninjaResult -eq $rninjaResult) {
            Write-Status "PASS: Both builders succeeded with exit code $ninjaResult" Green

            $appPath = @("myapp.exe", "myapp") |
                ForEach-Object { Join-Path $buildDir $_ } |
                Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } |
                Select-Object -First 1

            if ($appPath) {
                $output = (& $appPath).Trim()
                if ($output -eq "3") {
                    Write-Status "PASS: Executable produces correct output" Green
                }
                else {
                    Write-Status "FAIL: Executable output '$output' != '3'" Red
                    exit 1
                }
            }
            else {
                Write-Status "FAIL: Executable not found" Red
                exit 1
            }
        }
        else {
            Write-Status "FAIL: Exit codes differ (ninja=$ninjaResult, rninja=$rninjaResult)" Red
            exit 1
        }

        Write-Status "Testing incremental build..." Yellow
        (Get-Item -LiteralPath (Join-Path $srcDir "lib.c")).LastWriteTime = Get-Date
        [void](Invoke-QuietWithTimeout -FilePath $rninja -Arguments @("--no-daemon") -Environment @{ RNINJA_CACHE_ENABLED = "0" } -TimeoutSeconds 30)

        Write-Status "CMake generator test PASSED" Green
    }
    finally {
        Pop-Location
    }
}
finally {
    if (Test-Path -LiteralPath $testDir) {
        Remove-Item -LiteralPath $testDir -Recurse -Force
    }
}
