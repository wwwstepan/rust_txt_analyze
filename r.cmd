cls
C:\Users\Chaplin\.cargo\bin\cargo build
if %errorlevel% neq 0 goto finish
cd target\debug\
txt_analyze.exe 
cd ..\..\
:finish