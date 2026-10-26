::/*#! 2> /dev/null                                 #
@ 2>/dev/null # 2>nul & echo off & goto BOF         #
if [ -z ${SIREUM_HOME} ]; then                      #
  echo "Please set SIREUM_HOME env var"             #
  exit -1                                           #
fi                                                  #
exec ${SIREUM_HOME}/bin/sireum slang run "$0" "$@"  #
:BOF
setlocal
if not defined SIREUM_HOME (
  echo Please set SIREUM_HOME env var
  exit /B -1
)
%SIREUM_HOME%\bin\sireum.bat slang run "%0" %*
exit /B %errorlevel%
::!#*/
// #Sireum

import org.sireum._

val scriptDir = Os.slashDir
val cratesDir = scriptDir.up / "crates"

val SIREUM_HOME: Os.Path = Os.path(Os.env("SIREUM_HOME").get)
val sireum: Os.Path = SIREUM_HOME / "bin" / (if (Os.isWin) "sireum.bat" else "sireum")

//=============================================================================
//  Cross-platform helpers
//=============================================================================

// Run a shell command capturing output.
def sh(cmd: String): Os.Proc.Result = {
  if (Os.isWin) Os.proc(ISZ("cmd", "/c", cmd)).run()
  else Os.proc(ISZ("sh", "-c", cmd)).run()
}

// Run a shell command streaming output to the console; returns the exit code.
def shConsole(cmd: String): Z = {
  if (Os.isWin) Os.proc(ISZ("cmd", "/c", cmd)).console.run().exitCode
  else Os.proc(ISZ("sh", "-c", cmd)).console.run().exitCode
}

// Remove a directory tree.
def removeDir(dir: Os.Path): Unit = {
  if (dir.exists) {
    if (Os.isWin) Os.proc(ISZ("cmd", "/c", s"rmdir /s /q \"${dir.string}\"")).run()
    else Os.proc(ISZ("sh", "-c", s"rm -rf '${dir.string}'")).run()
  }
}

// Open a file in the default browser/viewer.
def openInBrowser(path: Os.Path): Unit = {
  if (Os.isWin) Os.proc(ISZ("cmd", "/c", "start", "", path.string)).run()
  else if (Os.isMac) Os.proc(ISZ("open", path.string)).run()
  else Os.proc(ISZ("xdg-open", path.string)).run()
}

// Check if a command is available on PATH.
def commandExists(name: String): B = {
  if (Os.isWin) Os.proc(ISZ("cmd", "/c", s"where ${name} >nul 2>nul")).run().exitCode == 0
  else Os.proc(ISZ("sh", "-c", s"which ${name} > /dev/null 2>&1")).run().exitCode == 0
}

// Shell path quoting (double quotes on Windows, single quotes on Unix).
def q(path: Os.Path): String = {
  if (Os.isWin) s"\"${path.string}\""
  else s"'${path.string}'"
}

// Stderr suppression redirect.
val suppress: String = if (Os.isWin) " 2>nul" else " 2>/dev/null"

// cd command (Windows needs /d for cross-drive).
def cdCmd(dir: Os.Path): String = {
  if (Os.isWin) s"cd /d \"${dir.string}\""
  else s"cd '${dir.string}'"
}

// Cargo test command with coverage instrumentation environment variables.
def coverageTestCmd(): String = {
  if (Os.isWin) "set CARGO_INCREMENTAL=0 && set RUSTFLAGS=-Cinstrument-coverage && set LLVM_PROFILE_FILE=target/coverage/cargo-test-%p-%m.profraw && cargo test"
  else "CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='target/coverage/cargo-test-%p-%m.profraw' cargo test"
}

//=============================================================================
//  Rust installation check
//=============================================================================

if (!commandExists("cargo")) {
  val rustInstaller = SIREUM_HOME / "bin" / "install" / "rust.cmd"
  println("Rust (cargo) not found. Running Sireum Rust installer ...")
  if (rustInstaller.exists) {
    Os.proc(ISZ(sireum.string, "slang", "run", rustInstaller.string)).console.run()
  } else {
    eprintln(s"Rust installer not found at: ${rustInstaller.string}")
    eprintln("Please install Rust manually: https://rustup.rs")
    Os.exit(z"1")
  }
}

val allComponentCrates: ISZ[String] = ISZ(
  "gen_gen",
  "splitter_splitter",
  "incx_incx",
  "decy_decy",
  "merger_merger",
  "consume_consume"
)

//=============================================================================
//  String helpers (Sireum String lacks * and isEmpty)
//=============================================================================

def strRepeat(ch: String, n: Z): String = {
  var r: String = ""
  for (_ <- z"0" until n) r = s"${r}${ch}"
  r
}

val SEP: String = strRepeat("=", z"70")

//=============================================================================
//  Coverage tooling detection
//=============================================================================

def checkCoverageAvailable(): B = {
  if (!commandExists("grcov")) {
    println("  [coverage] grcov not found on PATH -- coverage metrics disabled")
    println("             Install with: cargo install grcov")
    return F
  }
  val llvmResult = sh(s"rustup component list --installed${suppress}")
  val llvmOk = llvmResult.exitCode == 0 && ops.StringOps(llvmResult.out).contains("llvm-tools")
  if (!llvmOk) {
    println("  [coverage] llvm-tools-preview not installed -- coverage metrics disabled")
    println("             Enable with: rustup component add llvm-tools-preview")
    return F
  }
  T
}

val hasCoverage: B = checkCoverageAvailable()

//=============================================================================
//  Coverage analysis (pure Slang -- no awk or grep required)
//=============================================================================

// Parse a non-negative decimal integer string; returns None() on failure.
def parseNonNegZ(s: String): Option[Z] = {
  val t = ops.StringOps(s).trim
  if (t.size == z"0") return None()
  var result: Z = z"0"
  var ok: B = T
  for (i <- z"0" until t.size) {
    val ch = ops.StringOps(t).substring(i, i + z"1")
    if      (ch == "0") result = result * z"10"
    else if (ch == "1") result = result * z"10" + z"1"
    else if (ch == "2") result = result * z"10" + z"2"
    else if (ch == "3") result = result * z"10" + z"3"
    else if (ch == "4") result = result * z"10" + z"4"
    else if (ch == "5") result = result * z"10" + z"5"
    else if (ch == "6") result = result * z"10" + z"6"
    else if (ch == "7") result = result * z"10" + z"7"
    else if (ch == "8") result = result * z"10" + z"8"
    else if (ch == "9") result = result * z"10" + z"9"
    else ok = F
  }
  if (ok) Some(result) else None()
}

// Find the 1-based line number of the first occurrence of "pub fn <fnName>"
// in appFile.  Returns 0 when not found.
def findFnLine(appFile: Os.Path, fnName: String): Z = {
  val target = s"pub fn ${fnName}"
  val lines = appFile.readLines
  var result: Z = z"0"
  var i: Z = z"0"
  for (line <- lines) {
    i = i + z"1"
    if (result == z"0" && ops.StringOps(line).contains(target)) {
      result = i
    }
  }
  result
}

// Query per-method line coverage by parsing the lcov.info produced by grcov.
def methodCoverage(crateDir: Os.Path, crateName: String, startLine: Z, endLine: Z): String = {
  val lcov = crateDir / "target" / "coverage" / "lcov.info"
  if (!lcov.exists) return "no data"
  val appSuffix = s"src/component/${crateName}_app.rs"
  val lines = lcov.readLines
  var inFile: B = F
  var hit: Z = z"0"
  var total: Z = z"0"
  for (line <- lines) {
    if (ops.StringOps(line).startsWith("SF:")) {
      inFile = ops.StringOps(line).contains(appSuffix)
    } else if (ops.StringOps(line).startsWith("end_of_record")) {
      inFile = F
    } else if (inFile && ops.StringOps(line).startsWith("DA:")) {
      // DA:<line_number>,<execution_count>
      val rest = ops.StringOps(line).substring(z"3", line.size)
      val parts = ops.StringOps(rest).split(c => c == ',')
      if (parts.size >= z"2") {
        parseNonNegZ(parts(z"0")) match {
          case Some(lineNum) =>
            parseNonNegZ(parts(z"1")) match {
              case Some(hitCount) =>
                if (lineNum >= startLine && lineNum <= endLine) {
                  total = total + z"1"
                  if (hitCount > z"0") {
                    hit = hit + z"1"
                  }
                }
              case _ =>
            }
          case _ =>
        }
      }
    }
  }
  if (total == z"0") "N/A"
  else s"${hit * z"100" / total}% (${hit}/${total} lines)"
}

// Print coverage metrics for initialize and timeTriggered.
def printCoverageTable(crateDir: Os.Path, crateName: String): Unit = {
  val appFile = crateDir / "src" / "component" / s"${crateName}_app.rs"
  if (!appFile.exists) {
    println("  [coverage] app source file not found")
    return
  }

  val initLine   = findFnLine(appFile, "initialize")
  val ttLine     = findFnLine(appFile, "timeTriggered")
  val notifyLine = findFnLine(appFile, "notify")
  val lastLine: Z = appFile.readLines.size

  val initEnd: Z = if (ttLine     != z"0") ttLine - z"1"     else lastLine
  val ttEnd: Z   = if (notifyLine != z"0") notifyLine - z"1" else lastLine

  val initCov = if (initLine != z"0") methodCoverage(crateDir, crateName, initLine, initEnd)
                else "N/A (not found)"
  val ttCov   = if (ttLine   != z"0") methodCoverage(crateDir, crateName, ttLine, ttEnd)
                else "N/A (not found)"

  println()
  println("  Coverage:")
  println(s"    initialize    : ${initCov}")
  println(s"    timeTriggered : ${ttCov}")
}

// Generate the lcov coverage data file (quiet).
def generateLcov(crateDir: Os.Path): Unit = {
  sh(s"${cdCmd(crateDir)} && grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing -o target/coverage/lcov.info${suppress}")
}

// Generate the HTML coverage report.
def generateHtmlReport(crateDir: Os.Path): Unit = {
  sh(s"${cdCmd(crateDir)} && grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing -o target/coverage/report${suppress}")
}

// Run cargo test capturing all output quietly; caller inspects exitCode and out.
def runCargoTestWithCoverageQuiet(crateDir: Os.Path): Os.Proc.Result = {
  removeDir(crateDir / "target" / "coverage")
  sh(s"${cdCmd(crateDir)} && ${coverageTestCmd()} 2>&1")
}

def runCargoTestQuiet(crateDir: Os.Path): Os.Proc.Result = {
  sh(s"${cdCmd(crateDir)} && cargo test 2>&1")
}

// Run cargo test streaming all output to the console; returns exit code.
def runCargoTestWithCoverageVerbose(crateDir: Os.Path): Z = {
  removeDir(crateDir / "target" / "coverage")
  shConsole(s"${cdCmd(crateDir)} && ${coverageTestCmd()}")
}

def runCargoTestVerbose(crateDir: Os.Path): Z = {
  shConsole(s"${cdCmd(crateDir)} && cargo test")
}

// Extract failed test names from captured cargo test output.
// Looks for lines matching "test <name> ... FAILED".
def parseCargoTestFailures(out: String): ISZ[String] = {
  val lines = ops.StringOps(out).split(c => c == '\n')
  var failedTests: ISZ[String] = ISZ()
  val prefix: String = "test "
  val suffix: String = " ... FAILED"
  for (line <- lines) {
    val trimmed = ops.StringOps(line).trim
    if (ops.StringOps(trimmed).startsWith(prefix) &&
        ops.StringOps(trimmed).endsWith(suffix) &&
        trimmed.size > prefix.size + suffix.size) {
      val name = ops.StringOps(trimmed).substring(prefix.size, trimmed.size - suffix.size)
      failedTests = failedTests :+ name
    }
  }
  failedTests
}

//=============================================================================
//  Crate selection
//=============================================================================

// Returns crates to operate on given a list of substring filters.
// With no filters, returns all component crates.
def selectCrates(filters: ISZ[String]): ISZ[String] = {
  if (filters.isEmpty) return allComponentCrates
  var selected: ISZ[String] = ISZ()
  for (filter <- filters) {
    var found = F
    for (crate <- allComponentCrates) {
      var alreadyIn = F
      for (s <- selected) { if (s == crate) alreadyIn = T }
      if (ops.StringOps(crate).contains(filter) && !alreadyIn) {
        selected = selected :+ crate
        found = T
      }
    }
    if (!found) eprintln(s"Warning: no crate name contains '${filter}'")
  }
  selected
}

// Find exactly one crate matching a filter (needed for test-report).
def findSingleCrate(filter: String): Option[String] = {
  var matches: ISZ[String] = ISZ()
  for (crate <- allComponentCrates) {
    if (ops.StringOps(crate).contains(filter)) matches = matches :+ crate
  }
  if (matches.isEmpty) {
    eprintln(s"Error: no crate name contains '${filter}'")
    eprintln("Available crates:")
    for (c <- allComponentCrates) eprintln(s"  ${c}")
    None()
  } else if (matches.size > z"1") {
    eprintln(s"Error: '${filter}' matches multiple crates -- please be more specific:")
    for (c <- matches) eprintln(s"  ${c}")
    None()
  } else {
    Some(matches(z"0"))
  }
}

//=============================================================================
//  'test' command
//=============================================================================

def runTestsCmd(args: ISZ[String]): Unit = {
  // Parse 'verbose' flag and crate name filters from args.
  var verbose: B = F
  var filters: ISZ[String] = ISZ()
  for (a <- args) {
    if (a == "verbose") verbose = T
    else filters = filters :+ a
  }

  val crates = selectCrates(filters)
  if (crates.isEmpty) {
    eprintln("No crates selected.")
    return
  }

  println()
  if (!hasCoverage) {
    println("Note: coverage metrics unavailable.")
    println("      To enable: cargo install grcov && rustup component add llvm-tools-preview")
    println()
  }

  var anyFailed = F
  for (crate <- crates) {
    val crateDir = cratesDir / crate
    println()
    println(SEP)
    println(s"Crate: ${crate}")
    println(SEP)

    val exitCode: Z = if (verbose) {
      // Stream all output.
      if (hasCoverage) runCargoTestWithCoverageVerbose(crateDir)
      else             runCargoTestVerbose(crateDir)
    } else {
      // Capture output; show only on build failure.
      val r = if (hasCoverage) runCargoTestWithCoverageQuiet(crateDir)
              else             runCargoTestQuiet(crateDir)
      if (r.exitCode != z"0") {
        if (ops.StringOps(r.out).contains("test result:")) {
          val failedNames = parseCargoTestFailures(r.out)
          if (!failedNames.isEmpty) {
            println(s"  FAIL  ${failedNames.size} test(s) failed:")
            for (name <- failedNames) println(s"    - ${name}")
          } else {
            println("  FAIL  (re-run with 'verbose' to see test output)")
          }
        } else {
          print(r.out)  // compilation error -- always show
        }
      } else {
        println("  PASS")
      }
      r.exitCode
    }

    if (exitCode != z"0") anyFailed = T

    if (hasCoverage && exitCode == z"0") {
      generateLcov(crateDir)
      printCoverageTable(crateDir, crate)
    }
  }

  println()
  println(SEP)
  if (anyFailed) {
    if (verbose) println("RESULT: one or more crates had test failures  (see output above)")
    else         println("RESULT: one or more crates had test failures  (re-run with 'verbose' to see output)")
  } else {
    println("RESULT: all tests passed")
  }
  println(SEP)
}

//=============================================================================
//  'test-report' command
//=============================================================================

def runTestReportCmd(args: ISZ[String]): Unit = {
  if (!hasCoverage) {
    eprintln("Error: coverage tools required for test-report.")
    eprintln("       cargo install grcov && rustup component add llvm-tools-preview")
    return
  }

  // Parse 'fresh' and 'verbose' flags; remaining tokens are the crate filter.
  var isFresh: B = F
  var verbose: B = F
  var nameArgs: ISZ[String] = ISZ()
  for (a <- args) {
    if (a == "fresh") isFresh = T
    else if (a == "verbose") verbose = T
    else nameArgs = nameArgs :+ a
  }

  if (nameArgs.isEmpty) {
    eprintln("Usage: build.cmd test-report <crate-name-or-filter> [fresh] [verbose]")
    eprintln()
    eprintln("Available crates:")
    for (c <- allComponentCrates) eprintln(s"  ${c}")
    return
  }

  findSingleCrate(nameArgs(z"0")) match {
    case Some(crateName) =>
      val crateDir   = cratesDir / crateName
      val lcovFile   = crateDir / "target" / "coverage" / "lcov.info"
      val reportHtml = crateDir / "target" / "coverage" / "report" / "index.html"

      if (isFresh) {
        println(s"[fresh] Clearing previous coverage data for ${crateName} ...")
        removeDir(crateDir / "target" / "coverage")
      }

      val needRun = isFresh || !lcovFile.exists

      if (needRun) {
        println(s"Running tests with coverage for: ${crateName}")
        if (verbose) {
          println()
          runCargoTestWithCoverageVerbose(crateDir)
        } else {
          val r = runCargoTestWithCoverageQuiet(crateDir)
          if (r.exitCode != z"0") {
            if (ops.StringOps(r.out).contains("test result:")) {
              val failedNames = parseCargoTestFailures(r.out)
              if (!failedNames.isEmpty) {
                println(s"  FAIL  ${failedNames.size} test(s) failed:")
                for (name <- failedNames) println(s"    - ${name}")
              } else {
                println("  FAIL  (re-run with 'verbose' to see test output)")
              }
            } else {
              print(r.out)  // compilation error -- always show
            }
          }
        }
        println()
        println("Generating coverage data ...")
        generateLcov(crateDir)
        generateHtmlReport(crateDir)
        printCoverageTable(crateDir, crateName)
      } else {
        println(s"Coverage data exists. Use 'fresh' to re-run tests.")
        if (!reportHtml.exists) {
          println("Generating HTML report ...")
          generateHtmlReport(crateDir)
        }
        printCoverageTable(crateDir, crateName)
      }

      if (!reportHtml.exists) {
        eprintln("Error: HTML report was not generated. Check that grcov ran successfully.")
        return
      }

      println()
      println(s"Opening: ${reportHtml.string}")
      openInBrowser(reportHtml)

    case _ => // error already printed by findSingleCrate
  }
}

//=============================================================================
//  'clean' command
//=============================================================================

def runCleanCmd(): Unit = {
  for (crate <- allComponentCrates) {
    val crateDir = cratesDir / crate
    println(s"Cleaning: ${crate}")
    shConsole(s"${cdCmd(crateDir)} && make clean")
  }
}

//=============================================================================
//  Usage
//=============================================================================

def printUsage(): Unit = {
  println("Usage: build.cmd <command> [args...]")
  println()
  println("Commands:")
  println()
  println("  test [filter...] [verbose]")
  println("    Run unit tests for all component crates, or only those whose")
  println("    name contains at least one of the given filter strings.")
  println("    Build and test output are hidden by default; only PASS/FAIL is")
  println("    shown. Pass 'verbose' to stream full output.")
  println("    When grcov and llvm-tools-preview are available, per-method")
  println("    coverage metrics for initialize and timeTriggered are shown.")
  println()
  println("  test-report <filter> [fresh] [verbose]")
  println("    Open the HTML coverage report for the crate matched by <filter>.")
  println("    Tests are run automatically if no coverage data exists.")
  println("    Pass 'fresh' to delete old data, re-run tests, and rebuild the")
  println("    report from scratch.")
  println("    Pass 'verbose' to stream full build and test output.")
  println("    Requires grcov and llvm-tools-preview.")
  println()
  println("  clean")
  println("    Run 'make clean' for all component crates.")
  println()
  println("Component crates:")
  for (c <- allComponentCrates) println(s"  ${c}")
  println()
  println("Examples:")
  println("  build.cmd test")
  println("  build.cmd test <filter>")
  println("  build.cmd test <filter> verbose")
  println("  build.cmd test-report <filter>")
  println("  build.cmd test-report <filter> fresh")
  println("  build.cmd test-report <filter> fresh verbose")
  println("  build.cmd clean")
}

//=============================================================================
//  Entry point
//=============================================================================

val cliArgs: ISZ[String] = Os.cliArgs

if (cliArgs.isEmpty) {
  printUsage()
} else {
  cliArgs(z"0") match {
    case string"test"        => runTestsCmd(ops.ISZOps(cliArgs).drop(z"1"))
    case string"test-report" => runTestReportCmd(ops.ISZOps(cliArgs).drop(z"1"))
    case string"clean"       => runCleanCmd()
    case _                   => printUsage()
  }
}
