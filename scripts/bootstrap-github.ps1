param(
    [Parameter(Mandatory=$true)]
    [string]$Repository
)

$ErrorActionPreference = "Stop"
[Console]::InputEncoding=[Text.UTF8Encoding]::new($false)
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$OutputEncoding=[Text.UTF8Encoding]::new($false)

$labels = @(
    "type: foundation",
    "type: validation",
    "type: algorithm",
    "type: search",
    "type: integration",
    "type: architecture",
    "type: performance",
    "type: benchmark",
    "type: backend",
    "type: gpu",
    "type: docs",
    "type: research",
    "type: correctness",
    "type: ux",
    "type: output",
    "priority: high",
    "priority: medium",
    "priority: low",
    "milestone: v0.1",
    "milestone: v0.2",
    "milestone: v0.3",
    "milestone: v0.4",
    "milestone: v0.5",
    "milestone: v0.6"
)

foreach ($label in $labels) {
    gh label create $label --repo $Repository --color "ededed" --force | Out-Null
}

$milestones = @(
    @{ title = "v0.1"; description = "CPU single MVP" },
    @{ title = "v0.2"; description = "CPU multi backend" },
    @{ title = "v0.3"; description = "Benchmark framework" },
    @{ title = "v0.4"; description = "Backend selector and GPU stubs" },
    @{ title = "v0.5"; description = "CUDA search-only prototype" },
    @{ title = "v0.6"; description = "CUDA compute / AMD research branch" }
)

foreach ($milestone in $milestones) {
    $body = @{ title = $milestone.title; description = $milestone.description } | ConvertTo-Json
    try {
        $body | gh api "repos/$Repository/milestones" --method POST --input - | Out-Null
    } catch {
        Write-Host "milestone may already exist: $($milestone.title)"
    }
}

Write-Host "Labels and milestones bootstrapped for $Repository"
Write-Host "Create issues from docs/github-issues.md or the tracked issue upload command."
