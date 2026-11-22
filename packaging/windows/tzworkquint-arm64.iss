; Script generated for Inno Setup 6
; TZWorkQuint Windows Release Setup Script
; Vendor: TZLabs (https://tzlabs.in)

#define MyAppName "TZWorkQuint"
#define MyAppVersion "3.0.0"
#define MyAppPublisher "TZLabs"
#define MyAppURL "https://tzlabs.in"
#define MyAppExeName "tz-workquint.exe"

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
AppId={{D37E6B20-5A3E-4B02-8610-18D37F2C0B81}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\TZWorkQuint
DefaultGroupName=TZWorkQuint
AllowNoIcons=yes
LicenseFile=..\..\dist\TZWorkQuint-3.0.0-win-arm64\LICENSE.txt
OutputDir=..\..\dist
OutputBaseFilename=TZWorkQuint-3.0.0-Setup-arm64
SetupIconFile=..\..\dist\TZWorkQuint-3.0.0-win-arm64\assets\brand\tz-workquint.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog commandline
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
UninstallDisplayIcon={app}\bin\tz-workquint.exe
VersionInfoVersion=3.0.0.0
VersionInfoCompany=TZLabs
VersionInfoDescription=TZWorkQuint Productivity & Engineering Suite
VersionInfoCopyright=Copyright (C) 2026 TZLabs contributors
VersionInfoProductName=TZWorkQuint

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
; Primary Suite Binaries
Source: "..\..\dist\TZWorkQuint-3.0.0-win-arm64\bin\tz-workquint.exe"; DestDir: "{app}\bin"; Flags: ignoreversion
Source: "..\..\dist\TZWorkQuint-3.0.0-win-arm64\bin\tz-cellforge.exe"; DestDir: "{app}\bin"; Flags: ignoreversion
Source: "..\..\dist\TZWorkQuint-3.0.0-win-arm64\bin\tz-wordloom.exe"; DestDir: "{app}\bin"; Flags: ignoreversion
Source: "..\..\dist\TZWorkQuint-3.0.0-win-arm64\bin\tz-pdflex.exe"; DestDir: "{app}\bin"; Flags: ignoreversion
Source: "..\..\dist\TZWorkQuint-3.0.0-win-arm64\bin\tz-draftline.exe"; DestDir: "{app}\bin"; Flags: ignoreversion
Source: "..\..\dist\TZWorkQuint-3.0.0-win-arm64\bin\tz-slidespark.exe"; DestDir: "{app}\bin"; Flags: ignoreversion

; Brand Assets
Source: "..\..\dist\TZWorkQuint-3.0.0-win-arm64\assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs

; Offline User Guide & Documentation Portal
Source: "..\..\dist\TZWorkQuint-3.0.0-win-arm64\docs\*"; DestDir: "{app}\docs"; Flags: ignoreversion recursesubdirs createallsubdirs

; Legal & Release Notes
Source: "..\..\dist\TZWorkQuint-3.0.0-win-arm64\LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\dist\TZWorkQuint-3.0.0-win-arm64\README.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\dist\TZWorkQuint-3.0.0-win-arm64\THIRD_PARTY_NOTICES.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
; Start Menu Group
Name: "{group}\TZWorkQuint"; Filename: "{app}\bin\tz-workquint.exe"; WorkingDir: "{app}\bin"; IconFilename: "{app}\assets\brand\tz-workquint.ico"
Name: "{group}\TZ CellForge"; Filename: "{app}\bin\tz-cellforge.exe"; WorkingDir: "{app}\bin"; IconFilename: "{app}\assets\brand\tz-cellforge.ico"
Name: "{group}\TZ Wordloom"; Filename: "{app}\bin\tz-wordloom.exe"; WorkingDir: "{app}\bin"; IconFilename: "{app}\assets\brand\tz-wordloom.ico"
Name: "{group}\TZ PDFlex"; Filename: "{app}\bin\tz-pdflex.exe"; WorkingDir: "{app}\bin"; IconFilename: "{app}\assets\brand\tz-pdflex.ico"
Name: "{group}\TZ Draftline"; Filename: "{app}\bin\tz-draftline.exe"; WorkingDir: "{app}\bin"; IconFilename: "{app}\assets\brand\tz-draftline.ico"
Name: "{group}\TZ SlideSpark"; Filename: "{app}\bin\tz-slidespark.exe"; WorkingDir: "{app}\bin"; IconFilename: "{app}\assets\brand\tz-slidespark.ico"
Name: "{group}\TZWorkQuint User Guide"; Filename: "{app}\docs\user-guide\index.html"
Name: "{group}\{cm:UninstallProgram,TZWorkQuint}"; Filename: "{uninstallexe}"

; Desktop Shortcut
Name: "{autodesktop}\TZWorkQuint"; Filename: "{app}\bin\tz-workquint.exe"; WorkingDir: "{app}\bin"; IconFilename: "{app}\assets\brand\tz-workquint.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\bin\tz-workquint.exe"; Description: "{cm:LaunchProgram,TZWorkQuint}"; Flags: nowait postinstall skipifsilent

[Registry]
; Capabilities Registration for Windows Default Apps
Root: HKA; Subkey: "Software\TZLabs\TZWorkQuint\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "TZWorkQuint"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\TZLabs\TZWorkQuint\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "TZWorkQuint Productivity & Engineering Suite"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\TZLabs\TZWorkQuint\Capabilities\FileAssociations"; ValueType: string; ValueName: ".xlsx"; ValueData: "TZLabs.CellForge.1"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\TZLabs\TZWorkQuint\Capabilities\FileAssociations"; ValueType: string; ValueName: ".docx"; ValueData: "TZLabs.Wordloom.1"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\TZLabs\TZWorkQuint\Capabilities\FileAssociations"; ValueType: string; ValueName: ".pdf"; ValueData: "TZLabs.PDFlex.1"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\TZLabs\TZWorkQuint\Capabilities\FileAssociations"; ValueType: string; ValueName: ".dxf"; ValueData: "TZLabs.Draftline.1"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\TZLabs\TZWorkQuint\Capabilities\FileAssociations"; ValueType: string; ValueName: ".pptx"; ValueData: "TZLabs.SlideSpark.1"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "TZWorkQuint"; ValueData: "Software\TZLabs\TZWorkQuint\Capabilities"; Flags: uninsdeletevalue

; CellForge Registration (.xlsx)
Root: HKA; Subkey: "Software\Classes\Applications\tz-cellforge.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "TZ CellForge"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-cellforge.exe\DefaultIcon"; ValueType: string; ValueData: "{app}\assets\brand\tz-cellforge.ico,0"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-cellforge.exe\SupportedTypes"; ValueType: string; ValueName: ".xlsx"; ValueData: ""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-cellforge.exe\shell\open\command"; ValueType: string; ValueData: """{app}\bin\tz-cellforge.exe"" ""%1"""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.CellForge.1"; ValueType: string; ValueData: "TZ CellForge Spreadsheet"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.CellForge.1\DefaultIcon"; ValueType: string; ValueData: "{app}\assets\brand\tz-cellforge.ico,0"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.CellForge.1\shell\open\command"; ValueType: string; ValueData: """{app}\bin\tz-cellforge.exe"" ""%1"""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.CellForge.1\shell\open"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "TZ CellForge"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\.xlsx\OpenWithProgids"; ValueType: string; ValueName: "TZLabs.CellForge.1"; ValueData: ""; Flags: uninsdeletevalue

; Wordloom Registration (.docx)
Root: HKA; Subkey: "Software\Classes\Applications\tz-wordloom.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "TZ Wordloom"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-wordloom.exe\DefaultIcon"; ValueType: string; ValueData: "{app}\assets\brand\tz-wordloom.ico,0"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-wordloom.exe\SupportedTypes"; ValueType: string; ValueName: ".docx"; ValueData: ""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-wordloom.exe\shell\open\command"; ValueType: string; ValueData: """{app}\bin\tz-wordloom.exe"" ""%1"""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.Wordloom.1"; ValueType: string; ValueData: "TZ Wordloom Document"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.Wordloom.1\DefaultIcon"; ValueType: string; ValueData: "{app}\assets\brand\tz-wordloom.ico,0"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.Wordloom.1\shell\open\command"; ValueType: string; ValueData: """{app}\bin\tz-wordloom.exe"" ""%1"""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.Wordloom.1\shell\open"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "TZ Wordloom"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\.docx\OpenWithProgids"; ValueType: string; ValueName: "TZLabs.Wordloom.1"; ValueData: ""; Flags: uninsdeletevalue

; PDFlex Registration (.pdf)
Root: HKA; Subkey: "Software\Classes\Applications\tz-pdflex.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "TZ PDFlex"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-pdflex.exe\DefaultIcon"; ValueType: string; ValueData: "{app}\assets\brand\tz-pdflex.ico,0"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-pdflex.exe\SupportedTypes"; ValueType: string; ValueName: ".pdf"; ValueData: ""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-pdflex.exe\shell\open\command"; ValueType: string; ValueData: """{app}\bin\tz-pdflex.exe"" ""%1"""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.PDFlex.1"; ValueType: string; ValueData: "TZ PDFlex PDF Document"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.PDFlex.1\DefaultIcon"; ValueType: string; ValueData: "{app}\assets\brand\tz-pdflex.ico,0"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.PDFlex.1\shell\open\command"; ValueType: string; ValueData: """{app}\bin\tz-pdflex.exe"" ""%1"""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.PDFlex.1\shell\open"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "TZ PDFlex"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\.pdf\OpenWithProgids"; ValueType: string; ValueName: "TZLabs.PDFlex.1"; ValueData: ""; Flags: uninsdeletevalue

; Draftline Registration (.dxf)
Root: HKA; Subkey: "Software\Classes\Applications\tz-draftline.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "TZ Draftline"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-draftline.exe\DefaultIcon"; ValueType: string; ValueData: "{app}\assets\brand\tz-draftline.ico,0"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-draftline.exe\SupportedTypes"; ValueType: string; ValueName: ".dxf"; ValueData: ""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-draftline.exe\shell\open\command"; ValueType: string; ValueData: """{app}\bin\tz-draftline.exe"" ""%1"""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.Draftline.1"; ValueType: string; ValueData: "TZ Draftline Drawing"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.Draftline.1\DefaultIcon"; ValueType: string; ValueData: "{app}\assets\brand\tz-draftline.ico,0"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.Draftline.1\shell\open\command"; ValueType: string; ValueData: """{app}\bin\tz-draftline.exe"" ""%1"""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.Draftline.1\shell\open"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "TZ Draftline"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\.dxf\OpenWithProgids"; ValueType: string; ValueName: "TZLabs.Draftline.1"; ValueData: ""; Flags: uninsdeletevalue

; SlideSpark Registration (.pptx)
Root: HKA; Subkey: "Software\Classes\Applications\tz-slidespark.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "TZ SlideSpark"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-slidespark.exe\DefaultIcon"; ValueType: string; ValueData: "{app}\assets\brand\tz-slidespark.ico,0"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-slidespark.exe\SupportedTypes"; ValueType: string; ValueName: ".pptx"; ValueData: ""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\tz-slidespark.exe\shell\open\command"; ValueType: string; ValueData: """{app}\bin\tz-slidespark.exe"" ""%1"""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.SlideSpark.1"; ValueType: string; ValueData: "TZ SlideSpark Presentation"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.SlideSpark.1\DefaultIcon"; ValueType: string; ValueData: "{app}\assets\brand\tz-slidespark.ico,0"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.SlideSpark.1\shell\open\command"; ValueType: string; ValueData: """{app}\bin\tz-slidespark.exe"" ""%1"""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\TZLabs.SlideSpark.1\shell\open"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "TZ SlideSpark"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\.pptx\OpenWithProgids"; ValueType: string; ValueName: "TZLabs.SlideSpark.1"; ValueData: ""; Flags: uninsdeletevalue

[UninstallDelete]
; Safely remove installed files and cache; strictly avoids touching any user documents in personal folders
Type: files; Name: "{app}\*.log"

[Code]
// Open https://tzlabs.in exactly once in default browser after successful interactive installation
procedure CurStepChanged(CurStep: TSetupStep);
var
  ErrorCode: Integer;
begin
  if (CurStep = ssDone) and not WizardSilent then
  begin
    ShellExec('open', 'https://tzlabs.in', '', '', SW_SHOWNORMAL, ewNoWait, ErrorCode);
  end;
end;

// Open https://tzlabs.in exactly once in default browser after successful interactive uninstallation
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  ErrorCode: Integer;
begin
  if (CurUninstallStep = usPostUninstall) and not UninstallSilent then
  begin
    ShellExec('open', 'https://tzlabs.in', '', '', SW_SHOWNORMAL, ewNoWait, ErrorCode);
  end;
end;


