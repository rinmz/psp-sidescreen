; Extra steps of the PSP Sidescreen installer, run by the template Tauri generates.
;
; The installer is a classic Windows wizard (welcome, licence, folder, install, finish). These
; steps add what the program needs on the machine: the virtual display driver, and the USB driver
; for the PSP. Both are done by sidescreen-setup.exe, which is installed with the program.
; Putting the program on the PSP needs the PSP connected in a special mode, so the program's own
; Setup page walks the person through it the first time it opens.
;
; The exit codes are those of sidescreen-setup (see crates/sidescreen-setup/src/main.rs).

!define SETUP_EXE "$INSTDIR\resources\sidescreen-setup.exe"
!define REGISTRY_KEY "Software\PSP Sidescreen"

!macro NSIS_HOOK_POSTINSTALL
  ; --- The virtual display driver: the extra screen Windows shows for the PSP
  DetailPrint "Installing the virtual display driver"
  nsExec::ExecToLog '"${SETUP_EXE}" install-vdd "$INSTDIR\resources\drivers\vdd"'
  Pop $0
  ${If} $0 == 0
    ; We put it there, so we may take it away again. A driver that was already there may belong
    ; to another program, and is left alone.
    WriteRegDWORD HKLM "${REGISTRY_KEY}" "InstalledVirtualDisplay" 1
  ${ElseIf} $0 == 6
    DetailPrint "The virtual display driver was already installed"
  ${Else}
    DetailPrint "The virtual display driver could not be installed (code $0)"
    ${IfNot} ${Silent}
      MessageBox MB_ICONEXCLAMATION|MB_OK "The virtual display driver could not be installed (code $0).$\r$\n$\r$\nPSP Sidescreen can still show an existing monitor on the PSP, but cannot give it a screen of its own. You can run the installer again to retry."
    ${EndIf}
  ${EndIf}

  ; --- The USB driver for the PSP
  DetailPrint "Installing the USB driver for the PSP"
  nsExec::ExecToLog '"${SETUP_EXE}" install-psp-driver "$INSTDIR\resources\drivers\psp\sidescreen-psp.inf"'
  Pop $0
  ${If} $0 == 0
  ${OrIf} $0 == 6
    DetailPrint "The USB driver for the PSP is ready"
  ${ElseIf} $0 == 4
    DetailPrint "Windows would not accept the PSP driver: it is not signed yet"
    ${IfNot} ${Silent}
      MessageBox MB_ICONEXCLAMATION|MB_OK "Windows did not accept the USB driver for the PSP, because the driver is not signed yet.$\r$\n$\r$\nPSP Sidescreen is installed. Windows has to be allowed to use the driver once, by hand. The steps are in docs\PSP-DRIVER.md in the project, and take about two minutes."
    ${EndIf}
  ${Else}
    DetailPrint "The USB driver for the PSP could not be installed (code $0)"
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; The virtual display driver is shared with other programs, so it is removed only if this
  ; installer put it there, and only if the person agrees.
  ReadRegDWORD $0 HKLM "${REGISTRY_KEY}" "InstalledVirtualDisplay"
  ${If} $0 == 1
    ${IfNot} ${Silent}
      MessageBox MB_ICONQUESTION|MB_YESNO "Remove the virtual display driver as well?$\r$\n$\r$\nIt was installed with PSP Sidescreen. Choose No if another program uses it." IDNO keep_driver
    ${EndIf}
    DetailPrint "Removing the virtual display driver"
    nsExec::ExecToLog '"${SETUP_EXE}" uninstall-vdd'
    Pop $0
    keep_driver:
  ${EndIf}
  DeleteRegKey HKLM "${REGISTRY_KEY}"
!macroend
