[Setup]
AppName=Inkwell GPU Server
AppVersion=0.3.0
AppPublisher=Inkwell
AppPublisherURL=https://github.com/Louisdelez/CodingPromptes
DefaultDirName={autopf}\Inkwell GPU Server
DefaultGroupName=Inkwell
OutputBaseFilename=InkwellGPUServer-Setup
OutputDir=Output
Compression=lzma2
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
SetupIconFile=assets\logo-96.png
UninstallDisplayIcon={app}\inkwell-gpu-server.exe
PrivilegesRequired=lowest

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Files]
Source: "target\release\prompt-ai-server.exe"; DestDir: "{app}"; DestName: "inkwell-gpu-server.exe"; Flags: ignoreversion
Source: "assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs

[Icons]
Name: "{group}\Inkwell GPU Server"; Filename: "{app}\inkwell-gpu-server.exe"; WorkingDir: "{app}"
Name: "{group}\Uninstall Inkwell GPU Server"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Inkwell GPU Server"; Filename: "{app}\inkwell-gpu-server.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional icons:"

[Run]
Filename: "{app}\inkwell-gpu-server.exe"; Description: "Launch Inkwell GPU Server"; Flags: nowait postinstall skipifsilent
