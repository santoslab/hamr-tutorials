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
exec ${SIREUM_HOME}/bin/sireum slang run -n "$0" "$@"         #
:BOF
if not defined SIREUM_HOME (
  echo Please set SIREUM_HOME env var
  exit /B -1
)
%SIREUM_HOME%\bin\sireum.bat slang run -n "%0" %*
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

val osateDir = {
  Os.env("OSATE_HOME") match {
    case Some(s) => Os.path(s)
    case _ => appDir / s"osate${if (Os.isMac) ".app" else ""}"
  }
}

installOsateGumbo()

val buildEnv: ISZ[(String, String)] = ISZ(
  ("SIREUM_HOME", sireumHome.string),
  ("OSATE_HOME", osateDir.string))

val projects = ISZ(
  "HAMR-Slang-Tutorials-Example-00",
  "HAMR-Slang-Tutorials-Example-00-AADL-Refactored",
  "HAMR-Slang-Tutorials-Prod-Cons",
  "HAMR-Slang-Tutorials-Prod-Cons-AADL-only"
)
for (p <- projects) {
  val root = home / p
  if ((root / "aadl" / "bin").exists) {
    val ret = buildProject(root)
    assert(ret)
  }
}

val results = proc"git status -s".at(home).run()
if (results.out.size != 0){
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
    // everything zipped up okay so commit all the changes
    proc"git add $home".at(home).runCheck()
    Os.proc(ISZ[String]("git", "commit", "-m", "GITHUB ACTIONS: Updating repo due to change detection.  See commit diff for mor info")).at(home).runCheck()
    proc"git push".at(home).runCheck()
  } else {
    println(
      st"""Changes were detected and projects re-zipped.  You'll need to manually
          |commit and push these to github since this isn't a CI run""".render)
  }
} else {
  println("No changes detected")
}



object Helper {
  def buildProject(root: Os.Path): B = {
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
      root / "hamr" / "slang" / ".idea", 
      root / "hamr" / "slang" / "out")) 
      f.removeAll()
    val z7 = appDir / "7za"
    val zipFile = zipDir / s"${root.name}.zip"
    zipFile.removeAll()
    println()
    return proc"$z7 a -tzip $zipFile ${root.name}".echo.at(root.up).run().ok
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

  def installOsateGumbo(): B = {
    val versions = (codegenHome / "jvm" / "src" / "main" / "resources" / "phantom_versions.properties").properties

    val hamrJar = s"org.sireum.aadl.osate.hamr_${versions.get("org.sireum.aadl.osate.plugins.version_alt").get}.jar"
    val gumboJar = s"org.sireum.aadl.gumbo_${versions.get("org.sireum.aadl.gumbo.plugins.version_alt").get}.jar"

    val pluginsDir: Os.Path =
      if (Os.isMac) osateDir / "Contents" / "Eclipse" / "plugins"
      else osateDir / "plugins"

    var alreadyInstalled = F
    if (pluginsDir.exists) {
      val files = ops.ISZOps(pluginsDir.list.map((p: Os.Path) => p.name))
      alreadyInstalled = files.contains(hamrJar) && files.contains(gumboJar)
    }

    if (alreadyInstalled) {
      println(s"OSATE already up to date: $osateDir\n")
      return T
    } else {
      println("Installing Sireum plugins into OSATE, this will take a while ...")
      val result = proc"$sireum hamr phantom -u -o ${osateDir.value}".console.run()
      if (result.ok) {
        println(s"OSATE installed at ${osateDir}")
      } else {
        eprintln(result.err)
      }
      println()
      return result.ok
    }
  }
}