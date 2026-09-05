---
title: "Remote Control"
category: "scripting-tutorial"
source: "scriptingpageswiki.txt"
---

# Remote Control

**Remote Control** allows a player (remote-controller) to take control of another unit (drone, or remote-controlled) that is different from the local `player` unit.

- Only a **human player** can remote-control another unit.
- Players cannot remote-control other players.
- Only one unit can be controlled at the same time; remote control has to be terminated before controlling another unit.
- When a player takes control of a unit that is controlled by another player, the unit is transferred to the new owner and the previous player control is terminated.

The remote control is handled **locally**, which is why it is important that all arguments for `remoteControl`, `remoteControlled` and `isRemoteControlling` commands are **local**.

Exception is `remoteControl`'s *target* argument, for which locality **does not matter**, in which case the unit will be moved to the controlling player machine's locality.

When the player is controlling the unit, the unit is **local** to player.

## How To

### Take Control

Executing `remoteControl` transfers the movement control. In order to take full control, the camera also has to be transferred - using `switchCamera` and ideally **before** `remoteControl` usage; the point of view and the firing control are then transferred.

In some cases `remoteControl` is enough and the `cameraOn` transfer happens automatically.

```sqf
otherUnit switchCamera "INTERNAL";
player remoteControl otherUnit;
```

### Obtain Remote Control Information (since Arma 3 v2.14)

The `remoteControlled` command works for both cases and can return either the player controlling the provided UAV or the UAV controlled by the provided player, depending on the argument used.

> **Note:** Before Arma 3 v2.14 and `remoteControlled`, see `remoteControl` examples for a workaround.

#### Get the Remote Controller

```sqf
private _drone = remoteControlled player;
```

#### Get the Remote Controlled Unit

```sqf
private _player = remoteControlled _drone;
```

### Ensure Proper Disconnection

Remote Control termination can automatically transfer the camera back to the player but might need to be forced in some cases.

When player controls a unit and the unit dies, the control stays with that unit. After a short time, the dead unit is removed from its group. This will make `remoteControlled` on this unit return `objNull`, but `isRemoteControlling` for the controlling player will still return `true` and the player will remain stuck in remote control mode.

To exit this mode, the following can be used:

```sqf
player remoteControl objNull;
switchCamera player; // if needed

// remote control can also be terminated when only the drone is known
objNull remoteControl _drone;
```

