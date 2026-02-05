# EOL 语言示例批量测试脚本
# 测试所有 examples 目录下的 .eol 文件能否正确编译和运行

param(
    [switch]$Verbose,
    [switch]$KeepOutput
)

$ErrorActionPreference = "Stop"
$eolcPath = "./target/release/eolc.exe"

# 颜色定义
$Green = "`e[32m"
$Red = "`e[31m"
$Yellow = "`e[33m"
$Reset = "`e[0m"

Write-Host "=== EOL Language Example Test Suite ===" -ForegroundColor Cyan
Write-Host ""

# 检查编译器是否存在
if (-not (Test-Path $eolcPath)) {
    Write-Host "${Red}Error: eolc.exe not found at $eolcPath${Reset}" 
    Write-Host "Please build the compiler first: cargo build --release"
    exit 1
}

# 获取所有 .eol 文件
$eolFiles = Get-ChildItem -Path "examples" -Filter "*.eol" | Sort-Object Name

if ($eolFiles.Count -eq 0) {
    Write-Host "${Red}No .eol files found in examples/ directory${Reset}"
    exit 1
}

Write-Host "Found $($eolFiles.Count) example files to test"
Write-Host ""

$passed = 0
$failed = 0
$results = @()

foreach ($file in $eolFiles) {
    $baseName = $file.BaseName
    $sourcePath = $file.FullName
    $exePath = "examples/$baseName.exe"
    $irPath = "examples/$baseName.ll"
    
    Write-Host "Testing $baseName... " -NoNewline
    
    try {
        # 编译
        $compileOutput = & $eolcPath $sourcePath $exePath 2>&1
        $compileExitCode = $LASTEXITCODE
        
        if ($compileExitCode -ne 0) {
            throw "Compilation failed with exit code $compileExitCode`n$compileOutput"
        }
        
        # 检查 EXE 是否存在
        if (-not (Test-Path $exePath)) {
            throw "EXE file not generated"
        }
        
        # 运行
        $runOutput = & $exePath 2>&1
        $runExitCode = $LASTEXITCODE
        
        if ($runExitCode -ne 0) {
            throw "Execution failed with exit code $runExitCode`n$runOutput"
        }
        
        Write-Host "${Green}PASS${Reset}"
        $passed++
        
        $results += [PSCustomObject]@{
            Name = $baseName
            Status = "PASS"
            Output = ($runOutput -join "`n").Substring(0, [Math]::Min(100, ($runOutput -join "`n").Length))
            Error = ""
        }
        
        if ($Verbose) {
            Write-Host "  Output preview: $($runOutput[0..2] -join '; ')"
        }
        
    } catch {
        Write-Host "${Red}FAIL${Reset}"
        Write-Host "  Error: $_" -ForegroundColor Red
        $failed++
        
        $results += [PSCustomObject]@{
            Name = $baseName
            Status = "FAIL"
            Output = ""
            Error = $_.Exception.Message
        }
    } finally {
        # 清理
        if (-not $KeepOutput) {
            if (Test-Path $exePath) { Remove-Item $exePath -Force }
            if (Test-Path $irPath) { Remove-Item $irPath -Force }
        }
    }
}

Write-Host ""
Write-Host "=== Test Summary ===" -ForegroundColor Cyan
Write-Host "Total:  $($passed + $failed)"
Write-Host "${Green}Passed: $passed${Reset}"
if ($failed -gt 0) {
    Write-Host "${Red}Failed: $failed${Reset}"
}
Write-Host ""

# 详细结果
if ($failed -gt 0) {
    Write-Host "Failed tests:" -ForegroundColor Red
    $results | Where-Object { $_.Status -eq "FAIL" } | ForEach-Object {
        Write-Host "  - $($_.Name)"
        Write-Host "    Error: $($_.Error.Split("`n")[0])"
    }
}

exit $failed
