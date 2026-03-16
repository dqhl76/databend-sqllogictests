// Copyright 2021 Datafuse Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use clap::Parser;

// Add options when run sqllogictest, such as specific dir or file
#[derive(Parser, Debug, Clone)]
pub struct SqlLogicTestArgs {
    #[arg(
        long = "run",
        use_value_delimiter = true,
        value_delimiter = ',',
        conflicts_with_all = ["dir", "file", "skipped_dir", "skipped_file"],
        help = "Run sqllogictests by glob patterns. This is the recommended selector for new usage"
    )]
    pub run: Option<Vec<String>>,

    #[arg(
        long = "skip",
        use_value_delimiter = true,
        value_delimiter = ',',
        requires = "run",
        conflicts_with_all = ["dir", "file", "skipped_dir", "skipped_file"],
        help = "Skip sqllogictests by glob patterns. This is the recommended selector for new usage"
    )]
    pub skip: Option<Vec<String>>,

    // Choose suits to run
    #[arg(
        short = 'u',
        long = "suites",
        help = "Legacy suites root kept for compatibility. Existing selectors such as --run_dir, --run_file, --skip_dir and --skip_file are resolved from under this path. Prefer --run for new usage; --run does not depend on --suites",
        default_value = "tests/sqllogictests/suites"
    )]
    pub suites: String,

    // Set specific dir to run
    #[arg(
        short = 'd',
        long = "run_dir",
        help = "Legacy selector kept for compatibility. Run sqllogictests in a specific directory name found under --suites, for example 'base'. Prefer --run for new usage"
    )]
    pub dir: Option<String>,

    // Set specific test file to run
    #[arg(
        short = 'f',
        long = "run_file",
        help = "Legacy selector kept for compatibility. Run sqllogictests in a specific test file name found under --suites. Prefer --run for new usage"
    )]
    pub file: Option<String>,

    // Set specific dir to skip
    #[arg(
        short = 's',
        long = "skip_dir",
        help = "Legacy selector kept for compatibility. Skip sqllogictests in specific directory names found under --suites. Prefer --skip for new usage"
    )]
    pub skipped_dir: Option<String>,

    // Set specific file to skip
    #[arg(
        short = 'x',
        long = "skip_file",
        help = "Legacy selector kept for compatibility. Skip sqllogictests in specific test file names found under --suites. Prefer --skip for new usage"
    )]
    pub skipped_file: Option<String>,

    // Set handler to run tests
    #[arg(
        short = 'l',
        long = "handlers",
        use_value_delimiter = true,
        value_delimiter = ',',
        help = "Choose handlers to run tests, support mysql, http handler, the arg is optional. If use multiple handlers, please use \',\' to split them"
    )]
    pub handlers: Option<Vec<String>>,

    // If enable complete mode
    #[arg(
        short = 'c',
        long = "complete",
        default_missing_value = "true",
        help = "The arg is used to enable auto complete mode"
    )]
    pub complete: bool,

    // If close fast fail.
    #[arg(
        long = "no-fail-fast",
        default_missing_value = "true",
        help = "The arg is used to cancel fast fail"
    )]
    pub no_fail_fast: bool,

    #[arg(
        short = 'p',
        long = "parallel",
        default_value_t = 1,
        help = "The arg is used to set parallel number"
    )]
    pub parallel: usize,

    #[arg(
        long = "enable_sandbox",
        default_missing_value = "true",
        help = "The arg is used to enable sandbox_tenant"
    )]
    pub enable_sandbox: bool,

    #[arg(
        long = "debug",
        default_missing_value = "true",
        help = "The arg is used to enable debug mode which would print some debug messages"
    )]
    pub debug: bool,

    #[arg(
        long = "bench",
        default_missing_value = "true",
        help = "The arg is used to run benchmark instead of test"
    )]
    pub bench: bool,

    // Set specific the database to connect
    #[arg(
        long = "database",
        default_value = "default",
        help = "Specify the database to connect, the default database is 'default'"
    )]
    pub database: String,

    #[arg(
        long = "port",
        default_value = "8000",
        help = "The databend server http port"
    )]
    pub port: u16,
}
