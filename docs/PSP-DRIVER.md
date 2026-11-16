# The PSP's USB driver

PSP Sidescreen talks to the PSP over USB with Windows' own WinUSB driver. Windows has to be told
that the PSP is a device that uses it. The installer carries a small file that does that
(`drivers/psp/sidescreen-psp.inf`), but the file is not signed, and Windows refuses unsigned
drivers. So the first time, on each PC, the driver is installed by hand, once.

This takes about two minutes. It needs the PC to be started once with driver signature checking
turned off, which is a standard Windows option that lasts only until the next restart. The driver
stays installed afterwards, and the PC is as protected as before.

## Before you start

- Run the PSP Sidescreen installer first.
- If the PC uses **BitLocker** (many laptops do), have the recovery key at hand before restarting
  into the options below. Windows sometimes asks for it when the start-up options are used.
  Find it at https://account.microsoft.com/devices/recoverykey.

## 1. Put the program on the PSP

Open PSP Sidescreen. Its Setup page asks you to connect the PSP and choose **USB Connection** on
the PSP. Then press **Put it on my PSP**. This needs no driver, so it works right away.

When it says Done, press Circle on the PSP to leave USB Connection.

## 2. Restart with driver signature checking turned off

1. Open **Settings, System, Recovery**. Next to **Advanced startup** press **Restart now**.
   (Or hold Shift and choose Restart from the Start menu.)
2. On the blue screen choose **Troubleshoot, Advanced options, Startup Settings, Restart**.
3. When the list of numbered options appears, press **7** (or **F7**):
   *Disable driver signature enforcement.*

Windows starts as usual.

## 3. Install the driver

1. On the PSP open **PSP Sidescreen** (Game, Memory Stick). It counts down three seconds and
   starts USB mode by itself.
2. On the PC open **Device Manager** (right-click the Start button). Under **Other devices** or
   **Universal Serial Bus devices** there is a device named **PSP** with a yellow warning sign.
3. Right-click it, **Update driver**, **Browse my computer for drivers**, and choose the folder
   `C:\Program Files\PSP Sidescreen\resources\drivers\psp`. Press Next.
4. Windows says it cannot verify the publisher. Choose **Install this driver software anyway**.

The device is now called **PSP Sidescreen (USB)**. You never have to do this again on this PC: the
driver survives restarts, and restarting normally is fine.

## Check

Open PSP Sidescreen, go to Display and press **Start**. The Setup page also shows whether the
driver is ready.

## If it does not work

- The device must be on the screen in Device Manager: the program must be running on the PSP in
  USB mode, with a data cable (not a charge-only one).
- If Windows still says the driver is unsigned, the restart did not use option 7. Do step 2 again
  and watch for the numbered list.
