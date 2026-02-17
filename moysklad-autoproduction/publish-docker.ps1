# Script for building and publishing Docker image to GitHub Container Registry
# Usage: .\publish-docker.ps1 -Version "1.0.0" -GitHubUser "username"
# Example: .\publish-docker.ps1 -Version "1.0.0" -GitHubUser "myusername"

param(
    [string]$Version = "latest",
    [string]$GitHubUser = ""
)

$ErrorActionPreference = "Stop"

# Configuration
$ImageName = "moysklad-autoproduction"
$Registry = "ghcr.io"

# Try to get GitHub user from git remote if not specified
if ([string]::IsNullOrEmpty($GitHubUser)) {
    try {
        $remoteUrl = git remote get-url origin 2>$null
        if ($remoteUrl -match "github\.com[/:]([^/]+)") {
            $GitHubUser = $matches[1]
        }
    } catch {
        # Ignore error
    }
}

if ([string]::IsNullOrEmpty($GitHubUser)) {
    Write-Host "Error: Could not determine GitHub username" -ForegroundColor Red
    Write-Host "Usage: .\publish-docker.ps1 -Version '1.0.0' -GitHubUser 'username'" -ForegroundColor Yellow
    exit 1
}

# Full image name (must be lowercase for Docker)
$FullImageName = "${Registry}/$($GitHubUser.ToLower())/$($ImageName.ToLower())"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Build and Push Docker Image" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Registry: ${Registry}" -ForegroundColor White
Write-Host "Image: ${FullImageName}" -ForegroundColor White
Write-Host "Version: ${Version}" -ForegroundColor White
Write-Host "==========================================" -ForegroundColor Cyan

# Change to script directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

# Build Docker image
Write-Host ""
Write-Host "[1/3] Building Docker image..." -ForegroundColor Yellow
docker build -t "${FullImageName}:${Version}" -t "${FullImageName}:latest" .

if ($LASTEXITCODE -ne 0) {
    Write-Host "Error building Docker image" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "[2/3] Login to GitHub Container Registry..." -ForegroundColor Yellow
Write-Host "Note: You need a GitHub PAT with write:packages scope" -ForegroundColor Gray
Write-Host "Create token at: https://github.com/settings/tokens" -ForegroundColor Gray
Write-Host ""

# Request token
$Token = Read-Host "Enter GitHub Personal Access Token (hidden input)" -AsSecureString
$PlainToken = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($Token))

# Login to registry
$PlainToken | docker login $Registry -u $GitHubUser --password-stdin

if ($LASTEXITCODE -ne 0) {
    Write-Host "Error logging in to registry. Make sure you have a PAT with write:packages scope" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "[3/3] Pushing image..." -ForegroundColor Yellow

# Push image with version tag
docker push "${FullImageName}:${Version}"

if ($LASTEXITCODE -ne 0) {
    Write-Host "Error pushing image with tag ${Version}" -ForegroundColor Red
    exit 1
}

# Push image with latest tag
docker push "${FullImageName}:latest"

if ($LASTEXITCODE -ne 0) {
    Write-Host "Error pushing image with tag latest" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Green
Write-Host "Successfully published!" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green
Write-Host "Image available at:" -ForegroundColor White
Write-Host "  ${FullImageName}:${Version}" -ForegroundColor Cyan
Write-Host "  ${FullImageName}:latest" -ForegroundColor Cyan
Write-Host ""
Write-Host "To pull the image:" -ForegroundColor White
Write-Host "  docker pull ${FullImageName}:${Version}" -ForegroundColor Yellow
Write-Host ""
Write-Host "To run container:" -ForegroundColor White
Write-Host "  docker run -p 8084:8084 ${FullImageName}:${Version}" -ForegroundColor Yellow
Write-Host "==========================================" -ForegroundColor Green

# Clear token from memory
$PlainToken = $null
