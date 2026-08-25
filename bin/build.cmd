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
  "HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-DT-add-GUMBO-solution",

  "HAMR-SysMLv2-Rust-seL4-P-EDP-Example",
  "HAMR-SysMLv2-Rust-seL4-P-EDP-Prod-Cons-Example",
  "HAMR-SysMLv2-Rust-seL4-P-EDP-SNG-Example",

  "HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit",
  "HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit-add-Prop-yLEx-solution"
)

// SysMLv2 system-property projects: they do not commit codegen's attestation artifacts
val sysPropProjects = ISZ(
  "HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit",
  "HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit-add-Prop-yLEx-solution"
)

// SysMLv2 projects verified via project-level `make verus` (every component crate --
// plus, for the system-property projects, the system-property proof crate) instead of
// the single thermostat crate
val projectLevelVerusProjects = ISZ(
  "HAMR-SysMLv2-Rust-seL4-P-EDP-Example",
  "HAMR-SysMLv2-Rust-seL4-P-EDP-Prod-Cons-Example",
  "HAMR-SysMLv2-Rust-seL4-P-EDP-SNG-Example",
  "HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit",
  "HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit-add-Prop-yLEx-solution"
)

for (p <- projects) {
  val root = home / p
  if ((root / "aadl" / "bin").exists) {
    val ret = buildAadlProject(root)
    assert(ret)
  }
  if ((root / "sysmlv2" / "bin").exists) {
    if (Os.isLinux || Os.isMac) {
      val ret = buildSysmlProject(
        root = root,
        removeAttestation = ops.ISZOps(sysPropProjects).contains(p),
        systemLevelVerus = ops.ISZOps(projectLevelVerusProjects).contains(p))
      assert(ret)
    }
  }
}

if (Os.isLinux) {
  // CI checks INSPECTA-models out into the workspace, for the provers-env it
  // carries -- see .github/workflows/ci-linux.yml.  Nothing tracks or ignores it
  // here, so 'git status -s' below would report it as a change of its own and
  // 'git add $home' would commit the whole checkout.  Everything that needed it
  // has run by this point, so it goes.
  val inspectaModels = home / "INSPECTA-models"
  if (isCI() && inspectaModels.exists) {
    println(s"Removing the INSPECTA-models checkout at $inspectaModels")
    inspectaModels.removeAll()
  }

  // The archives are rebuilt on every run rather than only when 'git status -s'
  // reports something, because that check cannot see every way one goes out of
  // date: a zip carries files git is not tracking -- the crates' Cargo.lock are
  // gitignored and land in it -- so codegen output can change inside an archive
  // while the working tree stays clean, and the stale zip would then be the one
  // that ships.
  //
  // Rebuilding them all would be its own problem, since 7z records each entry's
  // modification time and a checkout gives every file a fresh one, so an archive
  // rebuilt from unchanged sources differs byte-for-byte and all twelve would be
  // committed on every run.  So each is compared by content -- the per-entry
  // path/size/CRC listing, which those timestamps do not enter -- and one whose
  // contents did not change is restored, leaving nothing for git to see.
  println("Rezipping the projects")
  val zipTool = appDir.up / "7zz"
  for (p <- projects) {
    val zipFile = zipDir / s"$p.zip"
    val before = zipContents(zipFile, zipTool)
    val ret = zipit(home / p)
    assert (ret, s"$p failed during zip")
    if (before.nonEmpty && before == zipContents(zipFile, zipTool)) {
      // Identical contents.  An untracked archive has nothing to restore from,
      // and is left as built.
      if (proc"git ls-files --error-unmatch $zipFile".at(home).run().ok) {
        proc"git checkout -- $zipFile".at(home).runCheck()
      }
    } else {
      println(s"  contents of $p.zip changed")
    }
  }
  println()

  val results = proc"git status -s".at(home).run()
  if (results.out.size != 0) {
    // Something has changed since the last codegen.  We'll accept those changes
    // since the results passed Tipe, compiled, and the unit tests passed.

    println("Detected the following changes:")
    println(results.out)
    println()

    // Only the caller that supplies branch_name pushes the changes.  More than
    // one workflow runs this build -- see .github/workflows -- and they would
    // otherwise each detect the same differences and race to push them to the
    // same branch, so the branch is named by the one workflow that owns the
    // update and left unset by the others.
    val branchOpt = Os.env("branch_name")
    if (isCI() && branchOpt.nonEmpty) {
      // everything zipped up okay so commit all the changes to the branch specified via the caller
      val branch = branchOpt.get
      proc"git checkout -b $branch".at(home).runCheck()
      proc"git add $home".at(home).runCheck()
      Os.proc(ISZ[String]("git", "commit", "-m", "GITHUB ACTIONS: Updating repo due to change detection.  See commit diff for more info")).at(home).runCheck()
      proc"git push --set-upstream origin $branch".at(home).runCheck()
    } else if (isCI()) {
      println(
        st"""Changes were detected and projects re-zipped, but branch_name is not set,
            |so they are not pushed -- the workflow that supplies it is the one that
            |updates the repository""".render)
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

  // the model directory is the sysmlv2 subdirectory holding the project's .sysml files
  // (its sibling directories are the shared aadl-lib, the helper scripts in bin, and
  // possibly a .slang cache)
  def findModelDir(sysmlv2Dir: Os.Path): Os.Path = {
    var candidates = ISZ[Os.Path]()
    for (d <- sysmlv2Dir.list if d.isDir && d.name != "aadl-lib" && d.name != "bin" && !ops.StringOps(d.name).startsWith(".")) {
      candidates = candidates :+ d
    }
    assert(candidates.size == 1, s"Expected exactly one model directory under $sysmlv2Dir, found ${candidates.size}")
    return candidates(0)
  }

  def buildSysmlProject(root: Os.Path, removeAttestation: B, systemLevelVerus: B): B = {
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

    var ok = buildMicrokitTree(
      root = root,
      microkitDir = hamrDir / "microkit",
      removeAttestation = removeAttestation,
      systemLevelVerus = systemLevelVerus,
      verify = T,
      test = T)

    // A tutorial that ships a starting point has hamr/microkit-initial beside
    // the hamr/microkit its steps build up to.  Both come from the same model --
    // codegen writes wherever --sel4-output-dir points -- so both are cleaned and
    // regenerated here rather than only the one, which is how the starting point
    // came to hold output from a codegen several releases old.
    //
    // It is built but neither verified nor tested.  Its entry points are the
    // skeletons a reader begins from, logging and returning, while the contracts
    // woven into them are the finished ones: 'make verus' answers with
    // 'postcondition not satisfied' for initialize and timeTriggered, and the
    // GUMBOX property tests fail for the same reason (the plain unit tests pass).
    // That it compiles is the part worth checking -- a reader has to be able to
    // build the tree before writing a line of it.
    val initialDir = hamrDir / "microkit-initial"
    if (ok && initialDir.exists) {
      ok = buildMicrokitTree(
        root = root,
        microkitDir = initialDir,
        removeAttestation = removeAttestation,
        systemLevelVerus = systemLevelVerus,
        verify = F,
        test = F)
    }

    return ok
  }

  // Clean, regenerate and build one of a project's generated Microkit trees.
  def buildMicrokitTree(root: Os.Path, microkitDir: Os.Path, removeAttestation: B,
                        systemLevelVerus: B, verify: B, test: B): B = {
    println(s"Processing $microkitDir")

    val sysmlv2Dir = root / "sysmlv2"

    var ret = run(s"Cleaning $microkitDir", F, proc"sireum slang run ${sysmlv2Dir / "bin" / "clean.cmd"} $microkitDir")

    if (ret == 0) {
      ret = run(s"Running HAMR codegen", F, proc"sireum slang run ${sysmlv2Dir / "bin" / "run-hamr.cmd"} --platform Microkit --sel4-output-dir $microkitDir".at(findModelDir(sysmlv2Dir)))
    }

    if (ret == 0) {
      if (removeAttestation) {
        // the system-property projects do not commit codegen's attestation artifacts
        (microkitDir / "attestation").removeAll()
      }

      ret = run("Building the image", F, proc"make RUST_MAKE_TARGET=build-release".at(microkitDir))

      if (ret == 0 && verify) {
        if (systemLevelVerus) {
          ret = run("Verifying the project's crates", F, proc"make verus".at(microkitDir))
        } else {
          val thermCrateDir = microkitDir / "crates" / "thermostat_thermostat"
          ret = run("Verifying thermostat", F, proc"make verus".at(thermCrateDir))
        }
      }

      if (ret == 0 && test) {
        ret = run("Running the microkit unit tests", F, proc"make test".at(microkitDir))
      }

      removeBuildArtifacts(microkitDir)
    }

    return ret == 0
  }

  def buildAadlProject(root: Os.Path): B = {
    println(s"Processing $root")
    def runit(cmd: String): B = {
      val results = proc"$cmd".env(buildEnv).at(root).echo.run()
      if (!results.ok) {
        println(results.out)
        cprint(T, results.err)
      }
      return results.ok
    }
    var ret = runit((root / "aadl" / "bin" / "clean.cmd").string)
    if (ret) {
      ret = runit((root / "aadl" / "bin" / "run-hamr.cmd").string)
    }
    if (ret) {
      ret = runit(s"$sireum proyek tipe ${root / "hamr" / "slang" }")
    }
    if (ret) {
      ret = runit(s"$sireum proyek test ${root / "hamr" / "slang" }")
    }

    println()
    return ret
  }

  /** The archive's contents, as the per-entry path/size/CRC listing 7z reports.
    *
    * Deliberately not a hash of the file: 7z stores each entry's modification
    * time, and a fresh checkout stamps every file with the time it was written,
    * so two archives built from identical sources do not match byte-for-byte.
    * The listing below holds nothing that varies that way.
    *
    * None() when the archive does not exist or cannot be read, which reads as
    * "no previous contents to compare against".
    *
    * zipTool is passed in rather than derived here: script-level vals are not in
    * scope inside this object.
    */
  def zipContents(zipFile: Os.Path, zipTool: Os.Path): Option[String] = {
    if (!zipFile.exists) {
      return None()
    }
    val r = proc"$zipTool l -slt $zipFile".run()
    if (!r.ok) {
      return None()
    }
    var entries = ISZ[String]()
    var inEntries = F
    for (line <- ops.StringOps(r.out).split(c => c == '\n')) {
      val l = ops.StringOps(line).trim
      if (l == "----------") {
        // everything before this describes the archive itself, including its
        // own path, which differs between a checked-in zip and a rebuilt one
        inEntries = T
      } else if (inEntries) {
        val lo = ops.StringOps(l)
        if (lo.startsWith("Path = ") || lo.startsWith("Size = ") || lo.startsWith("CRC = ")) {
          entries = entries :+ l
        }
      }
    }
    return Some(st"${(entries, "\n")}".render)
  }

  def zipit(root: Os.Path) : B = {
    println(s"Zipping $root")
    for (f <- ISZ(
      root / "hamr" / "slang" / ".bloop",
      root / "hamr" / "slang" / ".idea",
      root / "hamr" / "slang" / "out")) {
      f.removeAll()
    }
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