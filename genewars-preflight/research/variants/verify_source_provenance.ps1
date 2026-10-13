# Read-only remote metadata/header checks. Save summaries only; never game bytes/maps.
$ErrorActionPreference = 'Stop'
$reviewSources = @(
    @{Repo='mefistotelis/genewars-re-helpers'; Ref='a85d5c517b524aa4906a720330ed1435bf8b65cf'; Headers=@()},
    @{Repo='mefistotelis/genewars'; Ref='72ec4c4908d26304dd373132cb3187c855d4bf58'; Headers=@('bio/src/cvector.cpp','bio/include/bio_ver.h')}
)
$reviewResults = foreach ($reviewSource in $reviewSources) {
    $reviewBase = 'https://api.github.com/repos/' + $reviewSource.Repo
    $reviewMeta = Invoke-RestMethod -Uri $reviewBase
    $reviewCommit = Invoke-RestMethod -Uri ($reviewBase + '/commits/' + $reviewSource.Ref)
    $reviewTree = Invoke-RestMethod -Uri ($reviewBase + '/git/trees/' + $reviewCommit.sha + '?recursive=1')
    $reviewHeaders = foreach ($reviewFile in $reviewSource.Headers) {
        $reviewText = (Invoke-WebRequest -Uri ('https://raw.githubusercontent.com/' + $reviewSource.Repo + '/' + $reviewCommit.sha + '/' + $reviewFile)).Content
        $reviewHeader = ($reviewText -split "`n" | Select-Object -First 18) -join "`n"
        [ordered]@{
            path = $reviewFile
            git_blob_sha1 = ($reviewTree.tree | Where-Object {$_.path -eq $reviewFile}).sha
            header_license = if ($reviewHeader -match 'GNU General Public License' -and $reviewHeader -match 'version 2' -and $reviewHeader -match 'any later version') {'GPL-2.0-or-later'} else {'Unknown'}
            source_url = 'https://github.com/' + $reviewSource.Repo + '/blob/' + $reviewCommit.sha + '/' + $reviewFile
        }
    }
    [ordered]@{
        repository=$reviewSource.Repo; commit=$reviewCommit.sha; commit_time=$reviewCommit.commit.committer.date
        github_api_license=$reviewMeta.license; tree_entries=$reviewTree.tree.Count; tree_truncated=$reviewTree.truncated
        license_named_paths=@($reviewTree.tree | Where-Object {$_.path -match '(?i)(license|copying|copyright|notice)'} | ForEach-Object {$_.path})
        checked_headers=@($reviewHeaders)
        tree_url=$reviewBase + '/git/trees/' + $reviewCommit.sha + '?recursive=1'
    }
}
$reviewOutput = [ordered]@{observed_at_utc=(Get-Date).ToUniversalTime().ToString('o');sources=@($reviewResults);scope='Metadata and two license headers only; no commercial binary/symbol-map download, no source reuse.'}
$reviewOutput | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'source-provenance.json') -Encoding utf8
$reviewOutput | ConvertTo-Json -Depth 8
