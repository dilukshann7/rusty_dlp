#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif
#ifndef SourceRoot
  #define SourceRoot ".."
#endif
#ifndef BinaryPath
  #define BinaryPath SourceRoot + "\target\x86_64-pc-windows-msvc\release\rusty-dlp.exe"
#endif

[Setup]
AppId={{AA416D2B-70EB-40F4-9ABA-7B2368A7AC59}
AppName=rusty_dlp
AppVersion={#AppVersion}
AppPublisher=rusty_dlp contributors
AppPublisherURL=https://github.com/dilukshann7/rusty_dlp
AppSupportURL=https://github.com/dilukshann7/rusty_dlp/issues
DefaultDirName={localappdata}\Programs\rusty_dlp
DefaultGroupName=rusty_dlp
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
OutputDir={#SourceRoot}\dist
OutputBaseFilename=rusty_dlp-{#AppVersion}-windows-x64-setup
SetupIconFile={#SourceRoot}\assets\branding\icon.ico
UninstallDisplayIcon={app}\rusty-dlp.exe
LicenseFile={#SourceRoot}\LICENSE
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
WizardSmallImageFile={#SourceRoot}\assets\branding\wizard-small.bmp
WizardImageFile={#SourceRoot}\assets\branding\wizard-large.bmp
DisableProgramGroupPage=yes
CloseApplications=no
RestartApplications=no
UninstallDisplayName=rusty_dlp
VersionInfoProductName=rusty_dlp
VersionInfoDescription=rusty_dlp installer

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked

[Files]
Source: "{#BinaryPath}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceRoot}\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceRoot}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceRoot}\THIRD_PARTY.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceRoot}\assets\dependencies.json"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceRoot}\assets\branding\*"; DestDir: "{app}\branding"; Flags: ignoreversion

[Icons]
Name: "{group}\rusty_dlp"; Filename: "{app}\rusty-dlp.exe"
Name: "{autodesktop}\rusty_dlp"; Filename: "{app}\rusty-dlp.exe"; Tasks: desktopicon

; No automatic launch, PATH changes, managed-tool downloads, or user-data deletion.
