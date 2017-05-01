# release-party-br

Release party automation.

Designed to automate creating pull requests for releasing to production, release-party-br looks for repos in an 
organization and creates pull requests from `master` to `release` branch on each repo.  Useful when there's many 
repos ready for a production release.

<table>
    <tr>
        <td><strong>Linux / OS X</strong></td>
        <td><a href="https://travis-ci.org/matthewkmayer/release-party-BR" title="Travis Build Status"><img src="https://travis-ci.org/matthewkmayer/release-party-BR.svg?branch=master" alt="travis-badge"></img></a></td>
    </tr>
    <tr>
        <td><strong>Windows</strong></td>
        <td><a href="https://ci.appveyor.com/project/matthewkmayer/release-party-br" title="Appveyor Build Status"><img src="https://ci.appveyor.com/api/projects/status/gkiqfanbhjrhhh8v/branch/master?svg=true" alt="appveyor-badge"></img></a></td>
    </tr>
</table>


## Running

### Compile and run

`RP_GITHUBTOKEN=your_personal_token_here RP_ORGURL="https://api.github.com/orgs/ORGHERE/repos" cargo run`

### Compile then run

`cargo build`

`RP_GITHUBTOKEN=your_personal_token_here RP_ORGURL="https://api.github.com/orgs/ORGHERE/repos" ./target/debug/release-party-br`

### Required environment variables

`RP_GITHUBTOKEN` - a personal access token to Github

`RP_ORGURL` - URL of the organization's repo list

### Optional env vars

`RP_DRYRUN` - set to `true` to do a dry-run, which doesn't create pull requests, but lists the ones it would create.
EG: `RP_DRYRUN=true cargo run`.

### Optional: repo ignore list

The `ignoredrepos.toml` file can contain a list of repositories to ignore.  See [ignoredrepos.toml](ignoredrepos.toml) 
for an example.

### Optional: compile in release mode

Run `cargo build --release` to create a release binary.  This will run faster than a debug build.  The binary will be created at 
`./target/release/release-party-br`.