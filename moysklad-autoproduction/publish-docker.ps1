# Script for building and publishing Docker image to GitHub Container Registry
# Usage: .\publish-docker.ps1 -Version "1.0.0" -GitHubUser "username"
# Example: .\publish-docker.ps1 -Version "1.0.0" -GitHubUser "myusername"
#
# GitHub token can be stored in .env file:
#   GITHUB_TOKEN=your_github_pat
#   GITHUB_USER=your_username

param(
    [string]$Version = "latest",
    [string]$GitHubUser = "",
    [string]$GitHubToken = ""
)

$ErrorActionPreference = "Stop"

# Configuration
$ImageName = "moysklad-autoproduction"
$Registry = "ghcr.io"

# Change to script directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

# Load .env file if exists
$envFile = Join-Path $scriptDir ".env"
if (Test-Path $envFile) {
    Write-Host "Loading configuration from .env file..." -ForegroundColor Gray
    Get-Content $envFile | ForEach-Object {
        $line = $_.Trim()
        if ($line -and !$line.StartsWith("#") -and $line.Contains("=")) {
            $parts = $line.Split("=", 2)
            $key = $parts[0].Trim()
            $value = $parts[1].Trim()
            
            # Remove surrounding quotes
            if (($value.StartsWith('"') -and $value.EndsWith('"')) -or 
                ($value.StartsWith("'") -and $value.EndsWith("'"))) {
                $value = $value.Substring(1, $value.Length - 2)
            }
            
            # Set environment variable if not already set
            if (![string]::IsNullOrEmpty($value) -and [string]::IsNullOrEmpty([Environment]::GetEnvironmentVariable($key))) {
                [Environment]::SetEnvironmentVariable($key, $value, "Process")
            }
        }
    }
}

# Get GitHub user from parameters, environment, or git remote
if ([string]::IsNullOrEmpty($GitHubUser)) {
    $GitHubUser = [Environment]::GetEnvironmentVariable("GITHUB_USER")
}

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
    Write-Host "Or set GITHUB_USER in .env file" -ForegroundColor Yellow
    exit 1
}

# Get GitHub token from parameters or environment
if ([string]::IsNullOrEmpty($GitHubToken)) {
    $GitHubToken = [Environment]::GetEnvironmentVariable("GITHUB_TOKEN")
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

# Request token if not provided
if ([string]::IsNullOrEmpty($GitHubToken)) {
    Write-Host "Note: You need a GitHub PAT with write:packages scope" -ForegroundColor Gray
    Write-Host "Create token at: https://github.com/settings/tokens" -ForegroundColor Gray
    Write-Host "Or set GITHUB_TOKEN in .env file to avoid this prompt" -ForegroundColor Gray
    Write-Host ""
    
    $Token = Read-Host "Enter GitHub Personal Access Token (hidden input)" -AsSecureString
    $GitHubToken = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($Token))
} else {
    Write-Host "Using GITHUB_TOKEN from environment/.env" -ForegroundColor Gray
}

# Login to registry
$GitHubToken | docker login $Registry -u $GitHubUser --password-stdin

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
$GitHubToken = $null
