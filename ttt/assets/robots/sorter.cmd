@echo off
setlocal enabledelayedexpansion

:: Loop through all png files
for /R %%F in (row-*-column-*.png) do (
  :: Extract Y from the filename
  for /F "tokens=2 delims=-" %%A in ("%%~nF") do set "Y=%%A"
  
  :: Extract X from the filename
  for /F "tokens=4 delims=-." %%B in ("%%~nF") do set "X=%%B"

  :: Create the directory if it doesn't exist
  if not exist "robot_!Y!" mkdir "robot_!Y!"

  :: Rename and move the file
  move "%%F" "robot_!Y!\frame_!X!.png"
)

endlocal