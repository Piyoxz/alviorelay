# Sequential publisher respecting crates.io rate-limit token bucket
$crates = @("alvio-egress", "alvio-client-core", "alvio-ingress", "alvio-relay")
$env:PATH = "D:\Rust\cargo\bin;$env:PATH"

foreach ($crate in $crates) {
    while ($true) {
        $now = [System.DateTime]::UtcNow.ToString("yyyy-MM-dd HH:mm:ss")
        Write-Host "[$now UTC] Attempting to publish $crate..."
        
        $output = & cargo publish --allow-dirty -p $crate 2>&1 | Out-String
        Write-Host $output
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host ">>> SUCCESS: $crate has been published to crates.io!"
            Start-Sleep -Seconds 10
            break
        }
        
        if ($output -match "status 429 Too Many Requests.*after (.*? GMT)") {
            $untilStr = $Matches[1]
            Write-Host "Rate limited by crates.io. Window resets at $untilStr."
            
            while ([System.DateTime]::UtcNow -lt ([System.DateTime]::Parse($untilStr))) {
                $rem = [System.DateTime]::Parse($untilStr) - [System.DateTime]::UtcNow
                Write-Host "Waiting $($rem.Minutes)m $($rem.Seconds)s until $untilStr..."
                Start-Sleep -Seconds 30
            }
            Write-Host "Rate limit window passed. Retrying $crate now..."
            Start-Sleep -Seconds 5
        } else {
            Write-Host "Error occurred while publishing $crate, pausing 15s..."
            Start-Sleep -Seconds 15
        }
    }
}

Write-Host "========================================="
Write-Host "ALL REMAINING CRATES PUBLISHED TO CRATES.IO!"
Write-Host "========================================="
