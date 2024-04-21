# tello-autopilot

Autopilot for the DJI Tello, along with video and sensor data acquisition.

## Features

-   ~~DJI Tello autopilot functionality~~
-   Redistribution of video and sensor data from the drone to the local host
-   Sending commands to the drone (refer to the Tello SDK User Guide)

## Usage

1. Connect to the drone via WiFi.
1. Start the [tello-detection](https://github.com/drone-autopilot/tello-detection).
1. Start the tello-autopilot.
1. The drone will move automatically (refer to the tello-detection source code for details).
1. A GUI watchdog tool, [TelloWatchdog](https://github.com/drone-autopilot/TelloWatchdog), is available.

## Service Addresses

-   Send commands to the drone (TCP): `127.0.0.1:8989`
-   Receive JSON sensor data (state) from the drone (TCP): `127.0.0.1:8990`
-   Receive video from the drone (UDP): `127.0.0.1:*` (since this is a whitelist system, it is necessary to register addresses for each guest)
