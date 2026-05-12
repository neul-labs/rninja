#!/usr/bin/env pwsh
# Test rninja with CMake-generated Ninja files

<#
.SYNOPSIS
Tests rninja against a CMake-generated Ninja project.

.DESCRIPTION
Creates a small C project, configures it with CMake's Ninja generator, builds it
with Ninja, then cleans and builds it with rninja. On Windows, the script looks
for an MSYS2 clang.exe first, then gcc.exe, adds the compiler directory to PATH,
and passes the selected compiler to CMake.

If TestDir is not provided, the script creates a temporary test directory and
removes it after a successful run. If rninja fails, the test directory is kept
for debugging.

.PARAMETER TestDir
Directory where the test CMake project should be created. If omitted, a random
temporary directory is used. User-provided directories are preserved after the
test run.

.EXAMPLE
.\tests\generators\cmake_test.ps1

Runs the test in a temporary directory.

.EXAMPLE
.\tests\generators\cmake_test.ps1 -TestDir .\target\cmake-test-debug

Runs the test in the specified directory and leaves the generated project there.
#>

param(
    [string]$TestDir
)

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

function Write-ProcessOutput {
    param(
        [AllowEmptyString()]
        [string]$Stdout,

        [AllowEmptyString()]
        [string]$Stderr
    )

    if ($Stdout.Length -gt 0) {
        [Console]::Out.Write($Stdout)
    }

    if ($Stderr.Length -gt 0) {
        [Console]::Error.Write($Stderr)
    }
}

function Invoke-Quiet {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FilePath,

        [string[]]$Arguments = @(),

        [hashtable]$Environment = @{}
    )

    Write-Status "> $FilePath $(Join-CommandArguments -Arguments $Arguments)" Blue

    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo.FileName = $FilePath
    $process.StartInfo.Arguments = Join-CommandArguments -Arguments $Arguments
    $process.StartInfo.UseShellExecute = $false
    $process.StartInfo.WorkingDirectory = (Get-Location).ProviderPath
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
    Write-ProcessOutput -Stdout $stdoutTask.Result -Stderr $stderrTask.Result
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
    $process.StartInfo.WorkingDirectory = (Get-Location).ProviderPath
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
        $process.WaitForExit()
        $stdoutTask.Wait()
        $stderrTask.Wait()
        Write-ProcessOutput -Stdout $stdoutTask.Result -Stderr $stderrTask.Result
        return 124
    }

    $process.WaitForExit()
    $stdoutTask.Wait()
    $stderrTask.Wait()
    Write-ProcessOutput -Stdout $stdoutTask.Result -Stderr $stderrTask.Result
    return $process.ExitCode
}

function Find-WindowsCCompiler {
    $searchDirs = @(
        "C:\msys64\ucrt64\bin",
        "C:\msys64\clang64",
        "C:\msys64\clang64\bin",
        "C:\msys64\mingw64\bin",
        "C:\msys64\mingw32\bin"
    )

    foreach ($compilerName in @("clang.exe", "gcc.exe")) {
        foreach ($dir in $searchDirs) {
            $compilerPath = Join-Path $dir $compilerName
            if (Test-Path -LiteralPath $compilerPath -PathType Leaf) {
                return @{
                    Path = $compilerPath
                    Directory = $dir
                    Name = $compilerName
                }
            }
        }
    }

    return $null
}

$scriptDir = $PSScriptRoot
$projectRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)

$isWindowsPlatform = $env:OS -eq "Windows_NT"
$rninjaName = if ($isWindowsPlatform) { "rninja.exe" } else { "rninja" }
$rninja = Join-Path $projectRoot "target/release/$rninjaName"
$ninja = "ninja"
$cmakeConfigureArgs = @("-G", "Ninja")
$ninjaBuildArgs = @("-v")
$rninjaBuildArgs = @("--no-daemon", "-v")

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

if ($isWindowsPlatform) {
    $cCompiler = Find-WindowsCCompiler
    if (-not $cCompiler) {
        Write-Status "SKIP: no clang.exe or gcc.exe found in supported MSYS2 directories" Yellow
        exit 0
    }

    $env:Path = "$($cCompiler.Directory);$env:Path"
    $cmakeConfigureArgs += "-DCMAKE_C_COMPILER=$($cCompiler.Path)"
    Write-Status "Using C compiler: $($cCompiler.Path)" Yellow
}

$cmakeConfigureArgs += ".."

if (-not (Test-Path -LiteralPath $rninja -PathType Leaf)) {
    $cargoResult = Invoke-Quiet -FilePath "cargo" -Arguments @("build", "--release", "--manifest-path", (Join-Path $projectRoot "Cargo.toml"))
    if ($cargoResult -ne 0) {
        Write-Status "FAIL: cargo build failed with exit code $cargoResult" Red
        exit $cargoResult
    }
}

$userProvidedTestDir = -not [string]::IsNullOrWhiteSpace($TestDir)
$preserveTestDir = $false

if ($userProvidedTestDir) {
    $testDir = [System.IO.Path]::GetFullPath($TestDir)
}
else {
    $testDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
}

New-Item -ItemType Directory -Path $testDir -Force | Out-Null

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
    New-Item -ItemType Directory -Path $srcDir -Force | Out-Null

    "int add(int a, int b) { return a + b; }" | Set-Content -LiteralPath (Join-Path $srcDir "lib.c") -NoNewline -Encoding ascii

    @"
#include <stdio.h>
extern int add(int a, int b);
int main() { printf("%d\n", add(1, 2)); return 0; }
"@ | Set-Content -LiteralPath (Join-Path $srcDir "main.c") -NoNewline -Encoding ascii

    $buildDir = Join-Path $testDir "build"
    New-Item -ItemType Directory -Path $buildDir -Force | Out-Null
    Push-Location $buildDir

    tree .. /F | Write-Host

    try {
        Write-Status "Running cmake -G Ninja..." Yellow
        $cmakeResult = Invoke-Quiet -FilePath "cmake" -Arguments $cmakeConfigureArgs
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
        $ninjaResult = Invoke-Quiet -FilePath $ninja -Arguments $ninjaBuildArgs

        Write-Status "Building with rninja..." Yellow
        [void](Invoke-Quiet -FilePath $ninja -Arguments @("clean"))
        $rninjaResult = Invoke-Quiet -FilePath $rninja -Arguments $rninjaBuildArgs -Environment @{ RNINJA_CACHE_ENABLED = "0" }
        if ($rninjaResult -ne 0) {
            $preserveTestDir = $true
        }

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
        [void](Invoke-QuietWithTimeout -FilePath $rninja -Arguments $rninjaBuildArgs -Environment @{ RNINJA_CACHE_ENABLED = "0" } -TimeoutSeconds 30)

        Write-Status "CMake generator test PASSED" Green
    }
    finally {
        Pop-Location
    }
}
finally {
    if (Test-Path -LiteralPath $testDir) {
        if ($userProvidedTestDir -or $preserveTestDir) {
            Write-Status "Preserving test directory: $testDir" Yellow
        }
        else {
            Remove-Item -LiteralPath $testDir -Recurse -Force
        }
    }
}
