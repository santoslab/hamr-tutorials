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
%SIREUM_HOME%\\bin\\sireum.bat slang run "%0" %*
exit /B %errorlevel%
::!#*/
// #Sireum

import org.sireum._

val sysmlDir: Os.Path = Os.slashDir.up
val modelDir: Os.Path = sysmlDir / "sng"

val sireumBin: Os.Path = Os.path(Os.env("SIREUM_HOME").get) / "bin"
val sireum: Os.Path = sireumBin / (if(Os.isWin) "sireum.bat" else "sireum")

assert ((sysmlDir / "aadl-lib").exists, s"${sysmlDir / "aadl-lib"} not present")

// relative args, run from the model directory (the codegen output-dir is resolved
// against the current working directory)
var codegenArgs: ISZ[String] = ISZ(
  sireum.value, "hamr", "sysml", "codegen",
  "--sourcepath", "../aadl-lib:.",
)

codegenArgs = codegenArgs ++ Os.cliArgs

codegenArgs = codegenArgs ++ ISZ[String](
  "--output-dir", "../../hamr",
  "--workspace-root-dir", "../..",
  "SNG.sysml")

val results = Os.proc(codegenArgs).at(modelDir).echo.console.run()

// Running under windows results in 23 which is an indication 
// a platform restart was requested. Codegen completes 
// successfully and the cli app returns 0 so 
// not sure why this is being issued.
if(results.exitCode == 0 || results.exitCode == 23) {
  Os.exit(0)
} else {
  println(results.err)
  Os.exit(results.exitCode)
}
