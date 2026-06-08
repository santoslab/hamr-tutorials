::/*#! 2> /dev/null                                           #
@ 2>/dev/null # 2>nul & echo off & goto BOF                   #
if [ -f "$0.com" ] && [ "$0.com" -nt "$0" ]; then             #
  exec "$0.com" "$@"                                          #
fi                                                            #
rm -f "$0.com"                                                #
if [ -z ${SIREUM_HOME} ]; then                                #
  echo "Please set SIREUM_HOME env var"                       #
  exit -1                                                     #
fi                                                            #
exec ${SIREUM_HOME}/bin/sireum slang run "$0" "$@"         #
:BOF
if not defined SIREUM_HOME (
  echo Please set SIREUM_HOME env var
  exit /B -1
)
%SIREUM_HOME%\bin\sireum.bat slang run "%0" %*
exit /B %errorlevel%
::!#*/
// #Sireum

import org.sireum._
import Helper._

val homeBin: Os.Path = Os.slashDir
val home: Os.Path = homeBin.up
val zipDir = home / "zips"

val sireumHome =  Os.path(Os.env("SIREUM_HOME").get) 
val sireum: Os.Path = sireumHome / "bin" / (if (Os.isWin) "sireum.bat" else "sireum")
val appDir: Os.Path = sireumHome / "bin" / (if (Os.isMac) "mac" else if (Os.isWin) "win" else "linux")
val codegenHome = sireumHome / "hamr" / "codegen"

val buildEnv: ISZ[(String, String)] = ISZ(
  ("SIREUM_HOME", sireumHome.string))

val projects = ISZ(
  "HAMR-Slang-Tutorials-Example-00",
  "HAMR-Slang-Tutorials-Example-00-AADL-Refactored",
  "HAMR-Slang-Tutorials-Prod-Cons",
  "HAMR-Slang-Tutorials-Prod-Cons-AADL-only",

  "HAMR-SysMLv2-Rust-seL4-P-DP-Example",
  "HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-add-DT-solution",
  "HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-DT-add-GUMBO-solution"
)
for (p <- projects) {
  val root = home / p
  if ((root / "aadl" / "bin").exists) {
    val ret = buildAadlProject(root)
    assert(ret)
  }
  if ((root / "sysmlv2" / "bin").exists) {
    val ret = buildSysmlProject(root)
    assert(ret)
  }
}

if (Os.isLinux) {
  val results = proc"git status -s".at(home).run()
  if (results.out.size != 0) {
    // Something has changed since the last codegen.  We'll accept those changes
    // since the results passed Tipe, compiled, and the unit tests passed.

    println("Detected the following changes:")
    println(results.out)
    println()

    for (p <- projects) {
      val ret = zipit(home / p)
      assert (ret, s"$p failed during zip")
    }

    if (isCI()) {
      // everything zipped up okay so commit all the changes to the branch specified via the caller
      val branch = Os.env("branch_name").get
      proc"git checkout -b $branch".at(home).runCheck()
      proc"git add $home".at(home).runCheck()
      Os.proc(ISZ[String]("git", "commit", "-m", "GITHUB ACTIONS: Updating repo due to change detection.  See commit diff for more info")).at(home).runCheck()
      proc"git push --set-upstream origin $branch".at(home).runCheck()
    } else {
      println(
        st"""Changes were detected and projects re-zipped.  You'll need to manually
            |commit and push these to github since this isn't a CI run""".render)
    }
  } else {
    println("No changes detected")
  }
}



object Helper {
  def removeBuildArtifacts(d: Os.Path): Unit = {
    val removeNames = ops.ISZOps(ISZ("build", "out", "target"))
    val removeDirs = Os.Path.walk(d, T, F, p => p.isDir && removeNames.contains(p.name))
    for (d <- removeDirs) {
      d.removeAll()
    }
  }
  
  def run(title: String, verboseArg: B, proc: OsProto.Proc): Z = {
    println(s"$title ...")
    val r = (if (verboseArg) proc.console.echo else proc).run()
    if (!r.ok) {
      println(s"$title failed!")
      cprintln(F, r.out)
      cprintln(T, r.err)
    }
    return r.exitCode
  }

  def buildSysmlProject(root: Os.Path): B = {
    if (Os.env("MICROKIT_SDK").isEmpty) {
      println("MICROKIT_SDK environment variable not set")
      return F
    }

    println(s"Processing $root")

    // update sysml aadl libraries
    val sysmlv2Dir = root / "sysmlv2"
    val aadlLibsDir = sysmlv2Dir / "aadl-lib"
    val sysmlAadlLibsDir: Os.Path = Os.env("SYSML_AADL_LIBRARIES") match {
      case Some(s) => Os.path(s)
      case _ =>
        println("SYSML_AADL_LIBRARIES environement variable is not set")
        return F
    }
    aadlLibsDir.removeAll()
    sysmlAadlLibsDir.copyOverTo(aadlLibsDir)
    (aadlLibsDir / ".git").removeAll()
    (aadlLibsDir / ".gitattributes").removeAll()
    (aadlLibsDir / ".gitignore").removeAll()

    val hamrDir = root / "hamr"
    val microkitDir = hamrDir / "microkit"

    var ret = run(s"Cleaning $microkitDir", F, proc"sireum slang run ${sysmlv2Dir / "bin" / "clean.cmd"} $microkitDir")

    if (ret == 0) {
      ret = run(s"Running HAMR codegen", F, proc"sireum slang run ${sysmlv2Dir / "bin" / "run-hamr.cmd"} --platform Microkit".at(sysmlv2Dir / "isolette-simple"))
    }

    if (ret == 0) {
      ret = run("Building the image", F, proc"make RUST_MAKE_TARGET=build-release".at(microkitDir))

      if (ret == 0) {
        val thermCrateDir = microkitDir / "crates" / "thermostat_thermostat"
        ret = run("Verifying thermostat", F, proc"make verus".at(thermCrateDir))
      }

      if (ret == 0) {
        ret = run("Running the microkit unit tests", F, proc"make test".at(microkitDir))
      }

      removeBuildArtifacts(microkitDir)
    }

    return ret == 0
  }

  def buildAadlProject(root: Os.Path): B = {
    println(s"Processing $root")
    def run(cmd: String): B = {
      val results = proc"$cmd".env(buildEnv).at(root).echo.run()
      if (!results.ok) {
        println(results.out)
        cprint(T, results.err)
      }
      return results.ok
    }
    var ret = run((root / "aadl" / "bin" / "clean.cmd").string)
    if (ret) {
      ret = run((root / "aadl" / "bin" / "run-hamr.cmd").string)
    }
    if (ret) {
      ret = run(s"$sireum proyek tipe ${root / "hamr" / "slang" }")
    }
    if (ret) {
      ret = run(s"$sireum proyek test ${root / "hamr" / "slang" }")
    }

    println()
    return ret
  }

  def zipit(root: Os.Path) : B = {
    println(s"Zipping $root")
    for (f <- ISZ(
      root / "hamr" / "slang" / ".bloop",
      root / "hamr" / "slang" / ".idea", 
      root / "hamr" / "slang" / "out")) 
      f.removeAll()
    val z7 = appDir.up / "7zz"
    val zipFile = zipDir / s"${root.name}.zip"
    zipFile.removeAll()
    println()
    val results = proc"$z7 a -tzip $zipFile ${root.name}".echo.at(root.up).run()
    if (!results.ok) {
      println(results.err)
      return F
    }
    return T
  }

  def cloneRepo(repo: String, proj: String, location: Os.Path): B = {
    val ret: B = if (!location.exists) {
      proc"git clone --rec $repo/$proj $location".console.run().ok
    } else {
      Os.proc(ISZ("git", "pull")).at(location).console.run().ok
    }
    return ret
  }

  def isCI(): B = {
    return Os.env("GITLAB_CI").nonEmpty || Os.env("GITHUB_ACTIONS").nonEmpty || Os.env("BUILD_ID").nonEmpty
  }

}