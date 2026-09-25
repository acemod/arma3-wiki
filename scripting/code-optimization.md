---
title: "Code Optimisation"
category: "scripting-tutorial"
source: "scriptingpageswiki.txt"
---

# Code Optimisation

## Rules

Three rules: **Make it work → Make it readable → Optimise then**.

### Make It Work

- Write pseudocode first: "Get all units near the city, for each west soldier add 30% damage".
- Use `-showScriptErrors` startup parameter — errors cause slower execution or complete failure.
- Read Arma RPT (report) files for detailed error information.

### Make It Readable

- Use meaningful variable names (`_uniform`, not `_u`). `_i` is accepted for iteration.
- Avoid one-lining; readability > minor memory gains.
- Indent properly. Use line returns and spaces — space is free.
- Extract repeated logic into functions.
- Prefer `switch` over deep `if`/`else` chains. Break long functions into smaller ones.
- Use camelCase for readability.

Code comparison:

```sqf
// Before
_w=[]; {_w pushbackunique primaryweapon _x} foreach((allunits+alldeadmen) select{_x call BIS_fnc_objectside==east});

// After
_weaponNames = [];
_allUnitsAliveAndDead = allUnits + allDeadMen;
_allEastAliveAndDead = _allUnitsAliveAndDead select { _x call BIS_fnc_objectSide == east };
{ _weaponNames pushBackUnique primaryWeapon _x } forEach _allEastAliveAndDead;
```

#### Constants

Use `#define` for repeated hard-coded values (file-scoped only):

```sqf
#define BUFFER 1.053 // no semicolon
_a = _x + BUFFER;
_b = _y + BUFFER;
```

**Global constants** via `Description.ext` + `getMissionConfigValue` (searches top-to-bottom, so place definitions at the top):

```cpp
// Description.ext
var1 = 123;
var2 = "123";
var3[] = { 1, 2, 3 };
rand = __EVAL(random 999);
```

```sqf
// Usage
hint str getMissionConfigValue "var1"; // 123
```

### Optimise Then

- Prefer **private variables** over global ones.
- Avoid iterating the same array multiple times.
- Pre-compile scripts: `_myFunction = compile preprocessFileLineNumbers "myFile.sqf";` instead of repeated `execVM`.
- Shorten variable names to the scope needed: `{ _uniform = uniform _x; systemChat _uniform; } forEach _allOpforUnits;`

## Code Optimisation

> **Note:** Tests and benchmarks were done with the latest Arma 3 version at the time Arma 3 v1.82 with Tank DLC. Game engine performance may have changed since.
>
> Benchmark result in milliseconds (ms) is an average for **10000** iterations.

> - **Red** means you **must** change your ways today, or with us you will ride...
> - **Orange** means you may want to look at it if you are targeting pure performance
> - **Green** means the gain is little to insignificant. Going through your code for this replacement is not worth it. You **may** only consider it for future code.

### Scheduled and Unscheduled Environment

There are two code environment types: **scheduled** and **unscheduled**.

- A **scheduled** script has an execution time limit of **3 ms** before being suspended to the benefit of another script until its turn comes back. It is a bit slower than unscheduled, but **suspending** (`sleep`, `waitUntil`) is allowed.
- An **unscheduled** script is not watched and will run without limitations. It is recommended for time-critical scripts, but **suspending** (`sleep`, `waitUntil`) is **not** allowed!

### Variable Assignment (Orange)

```sqf
private _myVar = [33, 66] select false;                    // 0.0013 ms
private _myVar = if (false) then { 33; } else { 66; };     // 0.0020 ms
private "_myVar"; if (false) then { _myVar = 33; } else { _myVar = 66; };    // 0.0025 ms
```

### Lazy Evaluation (Orange)

In SQF the following code will evaluate every single condition, even if one fails:

```sqf
if (a && b && c) then {};
```

Even if `a` returns `false` (and thus the entire Boolean expression can no longer become `true`), `b` and `c` will still be executed and evaluated regardless.

To avoid this behaviour, one can either imbricate `if` statements or use **lazy evaluation**. The latter is done like so:

```sqf
if (a && { b && { c } }) then {};
```

In the example above, condition evaluation stops once any condition evaluates to `false`.

#### Influence on Semantics

Depending on the arrangement of the curly brackets, lazy evaluation can change the precedence (and therefore the semantics) of a condition:

| Expression | Condition Value (a, b, c) | Evaluated Conditions | Result |
|---|---|---|---|
| `a && b || c` | false, any, any | a, b, c | true |
| `a && {b} || {c}` | false, any, any | a, c | true |
| `a && {b || {c}}` | false, any, any | a | false |

#### Performance

Using lazy evaluation is not always the best way as it can both speed up and slow down the code, depending on the current condition being evaluated:

```sqf
["true  || { false  || {false}}", nil, 100000] call BIS_fnc_codePerformance;    // 0.00080 ms
["true  ||  {false} || {false} ", nil, 100000] call BIS_fnc_codePerformance;    // 0.00105 ms
["false ||   false  ||  false  ", nil, 100000] call BIS_fnc_codePerformance;    // 0.00123 ms
["true  ||   false  ||  false  ", nil, 100000] call BIS_fnc_codePerformance;    // 0.00128 ms
["false ||  {false} || {false} ", nil, 100000] call BIS_fnc_codePerformance;    // 0.00200 ms
```

### Concatenating Strings (Red)

`myString = myString + otherString` works fine for small strings, however the bigger the string gets the slower the operation becomes:

```sqf
myString = ""; for "_i" from 1 to 10000 do { myString = myString + "123" };    // 290 ms
```

The solution is to use a string array that you will concatenate later:

```sqf
strings = [];
for "_i" from 1 to 10000 do { strings pushBack "123" };
strings = strings joinString "";    // 30 ms
```

### Array Manipulation (Red)

#### Add Elements

New commands `append` and `pushBack` hold the best score.

```sqf
_array = [0,1,2,3]; _array append [4,5,6];                                    // 0.0020 ms
_array = [0,1,2,3]; _array = _array + [4,5,6];                                // 0.0023 ms
_array = [0,1,2,3]; { _array set [count _array, _x]; } forEach [4,5,6];       // 0.0080 ms
```

```sqf
_array = [0,1,2,3]; _array pushBack 4;                                        // 0.0016 ms
_array = [0,1,2,3]; _array = _array + [4];                                    // 0.0021 ms
_array = [0,1,2,3]; _array set [count _array, _x];                            // 0.0022 ms
```

#### Iterate Elements

**If `_x` is not required**, `for` is twice as fast as `forEach` - otherwise, use `forEach` (see results below).

```sqf
private _array = allUnits;                                                        // 64 units              256 units

// if the amount of loops is known
for "_i" from 0 to 63 do {};                                                      // 0.0120 ms             0.0460 ms
for "_i" from 0 to count _array -1 do {};                                         // 0.0170 ms             0.0480 ms
for "_i" from 0 to count _array -1 do { private _x = _array select _i };          // 0.0500 ms             0.1896 ms
for "_i" from 0 to count _array -1 do { private _x = array select _i; _x setDamage 0 };  // 0.107 ms        0.39 ms

{} forEach _array;                                                                // 0.0250 ms             0.098 ms
{ _x setDamage 0 } forEach _array;                                                // 0.0770 ms             0.312 ms

_array apply {};                                                                  // 0.0250 ms             0.098 ms
_array apply { _x setDamage 0 };                                                  // 0.0770 ms             0.312 ms

private _i = 0;
while { _i < 64 } do { _i = _i + 1; };                                            // 0.048 ms              0.200 ms

// counts array every loop
private _i = 0;
while { _i < count _array } do { _i = _i + 1; };                                  // 0.062 ms              0.247 ms

private _i = 0;
while { _i < 64 } do { private _x = _array select _i; _i = _i + 1; };             // 0.086 ms              0.42 ms
```

#### Remove Elements

```sqf
_array = [0,1,2,3]; _array deleteAt 0;                                            // 0.0015 ms
_array = [0,1,2,3]; _array set [0, objNull]; _array = _array - [objNull];        // 0.0038 ms
```

```sqf
_array = [0,1,2,3]; _array deleteRange [1, 2];                                    // 0.0018 ms
_array = [0,1,2,3]; { _array set [_x, objNull] } forEach [1,2]; _array = _array - [objNull];  // 0.0078 ms
```

### Multiplayer Recommendations (Red)

- Do not saturate the network with information: `publicVariable` or public `setVariable` shouldn't be used at high frequency, else **everyone's performance experience** is at risk!
- The server is supposed to have a good CPU and a lot of memory, use it: store functions, run them from it, send only the result to the clients.
- `publicVariable` and `setVariable` variable name length impacts network, be sure to send well-named, understandable variables.
- Use, use and use `remoteExec` & `remoteExecCall`. Ditch `BIS_fnc_MP` for good!

## Equivalent Commands Performance

### call (Orange)

`call` without arguments is faster than `call` with arguments:

```sqf
call {};                    // 0.0007 ms
123 call {};                // 0.0013 ms
```

Since the variables defined in the parent scope will be available in the `call`ed child scope, it could be possible to speed up the code by avoiding passing arguments all together, for example writing:

```sqf
player addEventHandler ["HandleDamage", { call my_fnc_damage }];
```

instead of:

```sqf
player addEventHandler ["HandleDamage", { _this call my_fnc_damage }];
```

### execVM and call (Red)

> **Note:** Using `execVM` multiple times makes the game read the file and recompile it every time.
> If you use the script more than once, store its code in a variable or better, make it a Function!

```sqf
// myFile.sqf is an EMPTY file
private _myFunction = compile preprocessFileLineNumbers "myFile.sqf"; // compile time is done only once
call _myFunction;                    // 0.0009 ms
execVM "myFile.sqf";                 // 0.275 ms

// myFile.sqf is BIS_fnc_showRespawnMenu
private _myFunction = compile preprocessFileLineNumbers "myFile.sqf"; // compile time is done only once
["close"] call _myFunction;          // 0.0056 ms
["close"] execVM "myFile.sqf";       // 0.506 ms
```

### loadFile, preprocessFile and preprocessFileLineNumbers (Orange)

```sqf
// myFile.sqf is an empty file
loadFile "myFile.sqf";                    // 0.219 ms
preprocessFile "myFile.sqf";              // 0.353 ms
preprocessFileLineNumbers "myFile.sqf";   // 0.355 ms
```

```sqf
// myFile.sqf is BIS_fnc_showRespawnMenu
loadFile "myFile.sqf";                    // 0.3516 ms
preprocessFile "myFile.sqf";              // 2.75 ms
preprocessFileLineNumbers "myFile.sqf";   // 2.73 ms
```

```sqf
// myFile.sqf is a missing file
loadFile "myFile.sqf";                    // 0.692 ms
preprocessFile "myFile.sqf";              // 0.6225 ms
preprocessFileLineNumbers "myFile.sqf";   // 0.6225 ms
```

> **Note:** The comparison of `loadFile` with `preprocessFile*` is not exactly fair as `loadFile` doesn't preprocess the file's content. On the other hand, the loaded file cannot contain any `//` or `/* */` comments nor any preprocessor instructions (including debug information like line numbers or file information).

### if (Green)

```sqf
if (condition) then { /* thenCode */ };                                    // 0.0011 ms
if (condition) exitWith { /* exitCode */ };                                 // 0.0014 ms
if (condition) then { /* thenCode */ } else { /* elseCode */ };             // 0.0015 ms
if (condition) then [{ /* thenCode */ }, { /* elseCode */ }];              // 0.0016 ms
```

### if and select (Green)

Use `[array] select boolean` instead of the lazy-evaluated `if`.

```sqf
_result = ["false result", "true result"] select true;                      // 0.0011 ms
_result = if (true) then { "true result"; } else { "false result"; };      // 0.0017 ms
```

### if and switch (Orange)

```sqf
_result = call {
    if (false) exitWith {};
    if (false) exitWith {};
    if (true)  exitWith {};
    if (false) exitWith {};
    if (false) exitWith {};
};                            // 0.0032 ms
```

```sqf
_result = switch (true) do {
    case (false): {};
    case (false): {};
    case (true) : {};
    case (false): {};
    case (false): {};
};                            // 0.0047 ms
```

### if else and switch (Green)

```sqf
_mode = "killed";

switch _mode do {
    case "init": {};
    case "killed": {};
    case "respawned": {};
};                            // 0.0019 ms
```

```sqf
_mode = "killed";

if (_mode == "init") then {} else {
    if (_mode == "killed") then {} else {
        if (_mode == "respawned") then {};
    };
};                            // 0.0019 ms
```

### in vs find (Orange)

```sqf
// String search
"bar" in "foobar"                                            // 0.0008 ms
"foobar" find "bar" > -1                                     // 0.0012 ms
```

```sqf
// array search - case-sensitive
"bar" in ["foo", "Bar", "bar", "BAR"];                       // 0.0012 ms
["foo", "Bar", "bar", "BAR"] find "bar" > -1;                // 0.0016 ms
```

### for (Orange)

The `for "_i" from 0 to 10 do {}` is twice as fast as its alternative syntax, `for [{ _i = 0 }, { _i < 100 }, { _i = _i + 1 }] do {}`.

```sqf
for "_i" from 0 to 10 do { /* forCode */ };                                    // 0.015 ms
for [{ _i = 0 }, { _i < 100 }, { _i = _i + 1 }] do { /* forCode */ };         // 0.030 ms
```

### forEach vs count vs findIf (Green)

Both `forEach` and `count` commands will step through **all** the array elements and both commands will contain reference to current element with the `_x` variable.

However, `count` loop is a little faster than `forEach` loop, but it does not benefit from the `_forEachIndex` variable. Also, there is a limitation as the code inside `count` expects Boolean or Nothing while the command itself returns Number. This limitation is very important if you try to replace your `forEach` by `count`. If you have to add an extra `true`/`false`/`nil` at the end to make `count` work, it will be slower than the `forEach` equivalent.

```sqf
{ diag_log _x } count   [1,2,3,4,5];                    // 0.082 ms
{ diag_log _x } forEach [1,2,3,4,5];                     // 0.083 ms
```

```sqf
// with an empty array
_someoneIsNear = (allUnits findIf { _x distance [0,0,0] < 1000 }) != -1;    // 0.0046 ms
_someoneIsNear = { _x distance [0,0,0] < 1000 } count allUnits > 0;         // 0.0047 ms
_someoneIsNear = {
    if (_x distance [0,0,0] < 1000) exitWith { true };
    false
} forEach allUnits;                                                          // 0.0060 ms
```

```sqf
// with a 30 items array
_someoneIsNear = (allUnits findIf { _x distance [0,0,0] < 1000 }) != -1;    // 0.0275 ms
_someoneIsNear = { _x distance [0,0,0] < 1000 } count allUnits > 0;         // 0.0645 ms
_someoneIsNear = {
    if (_x distance [0,0,0] < 1000) exitWith { true };
    false
} forEach allUnits;                                                          // 0.0390 ms
```

### findIf (Red)

`findIf` stops array iteration as soon as the condition is met.

```sqf
[0,1,2,3,4,5,6,7,8,9] findIf { _x == 2 };                                    // 0.0050 ms
{ if (_x == 2) exitWith { _forEachIndex; }; } forEach [0,1,2,3,4,5,6,7,8,9]; // 0.0078 ms
_quantity = { _x == 2 } count [0,1,2,3,4,5,6,7,8,9];                         // 0.0114 ms
```

### format vs str (Green)

```sqf
str 33;                // 0.0016 ms
format ["%1", 33];     // 0.0022 ms
```

### + vs format vs joinString (Green)

**Non-string data:**

```sqf
[33, 45, 78] joinString "";            // 0.0052 ms - no length limit
format ["%1%2%3", 33, 45, 78];         // 0.0054 ms - limited to ~8Kb
str 33 + str 45 + str 78;              // 0.0059 ms - no length limit
```

**String data:**

```sqf
["str1", "str2", "str3"] joinString "";    // 0.0015 ms - no length limit
format ["%1%2%3", "str1", "str2", "str3"]; // 0.0015 ms - limited to ~8Kb
"str1" + "str2" + "str3";                  // 0.0012 ms - no length limit
```

### private (Orange)

Direct declaration (`private _var = value`) is faster than declaring **then** assigning the variable.

```sqf
private _a = 1;
private _b = 2;
private _c = 3;
private _d = 4;
// 0.0023 ms
```

```sqf
private ["_a", "_b", "_c", "_d"];
_a = 1;
_b = 2;
_c = 3;
_d = 4;
// 0.0040 ms
```

However, if you have to reuse the same variable in a loop, external declaration is faster. The reason behind this is that a declaration in the loop will create, assign and delete the variable in each loop. An external declaration creates the variable only once and the loop only assigns the value.

```sqf
private ["_a", "_b", "_c", "_d"];
for "_i" from 1 to 10 do
{
    _a = 1; _b = 2; _c = 3; _d = 4;
};
// 0.0195 ms
```

```sqf
for "_i" from 1 to 10 do
{
    private _a = 1; private _b = 2; private _c = 3; private _d = 4;
};
// 0.0235 ms
```

### isNil (Orange)

```sqf
isNil "varName";        // 0.0007 ms
isNil { varName };      // 0.0012 ms
```

### isEqualType and typeName (Red)

`isEqualType` is much faster than `typeName`

```sqf
"string" isEqualType 33;                    // 0.0006 ms
typeName "string" == typeName 33;           // 0.0018 ms
```

### isEqualTo and count (Green)

```sqf
// with a items array
allUnits isEqualTo [];            // 0.0040 ms
count allUnits == 0;             // 0.0043 ms
```

### select and param (Green)

```sqf
[1,2,3] select 0;        // 0.0008 ms
[1,2,3] param [0];       // 0.0011 ms
```

### createSimpleObject vs createVehicle (Red)

```sqf
// createSimpleObject is over 43x faster than createVehicle!
deleteVehicle createSimpleObject ["a3\structures_f_mark\vr\shapes\vr_shape_01_cube_1m_f.p3d", [0,0,0]];  // ~0.08 ms
deleteVehicle createSimpleObject ["Land_VR_Shape_01_cube_1m_F", [0,0,0]];                                // ~3.2 ms
deleteVehicle createVehicle ["Land_VR_Shape_01_cube_1m_F", [0,0,0], [], 0, "CAN_COLLIDE"];               // ~2.7 ms
deleteVehicle createVehicle ["Land_VR_Shape_01_cube_1m_F", [0,0,0], [], 0, "NONE"];                      // ~78 ms
```

### objectParent and vehicle (Orange)

```sqf
isNull objectParent player;        // 0.0013 ms
vehicle player == player;          // 0.0022 ms
```

### nearEntities and nearestObjects (Red)

`nearEntities` is much faster than `nearestObjects` given on range and amount of objects within the given range. If range is over 100 meters it is highly recommended to use `nearEntities` over `nearestObjects`.

```sqf
// tested with a NATO rifle squad amongst solar power plant panels on Altis at coordinates [20762,15837]
getPosATL player nearEntities [["CAManBase"], 50];        // 0.0075 ms
nearestObjects [getPosATL player, ["CAManBase"], 50];     // 0.0145 ms
```

> **Note:** `nearEntities` only searches for **alive** objects and on-foot soldiers. In-vehicle units, killed units, destroyed vehicles, static objects and buildings will be ignored.

### Global Variables vs Local Variables (Orange)

If you need to use global variable repeatedly in a loop, copy its value to local variable and use local variable instead:

```sqf
SomeGlobalVariable = [123];
for "_i" from 1 to 100 do
{
    SomeGlobalVariable select 0;
};
// 0.13 ms
```

is noticeably slower than

```sqf
SomeGlobalVariable = [123];
private _var = SomeGlobalVariable;
for "_i" from 1 to 100 do
{
    _var select 0;
};
// 0.08 ms
```

### Config path delimiter (Green)

`>>` is slightly faster than `/` when used in config path with `configFile` or `missionConfigFile`.

```sqf
 configFile >> "CfgVehicles";        // 0.0019 ms
 configFile  / "CfgVehicles";        // 0.0023 ms
```

> **Note:** A config path can be stored in a variable for later use, saving CPU time: `_cfgVehicles = configFile >> "CfgVehicles"`.

### getPos* and setPos* (Orange)

```sqf
getPosWorld                     // 0.0015 ms
getPosASL                       // 0.0016 ms
getPosATL                       // 0.0016 ms
getPosASLW                      // 0.0023 ms
getPos                          // 0.0030-0.0300 ms; performance depends on where this command is used
position                        // same as getPos
getPosVisual                    // same as getPos
visiblePosition                 // same as getPos
```

```sqf
setPosWorld                     // 0.0060 ms
setPosASL                       // 0.0060 ms
setPosATL                       // 0.0060 ms
setPos                          // 0.0063 ms
setPosASLW                      // 0.0068 ms
setVehiclePosition              // 0.0077 ms with "CAN_COLLIDE"
                                // 0.0390 ms with "NONE"
```

### toLower/toUpper vs toLowerANSI/toUpperANSI (Orange)

```sqf
// _myString is a 100 chars "aAaAaA(...)" string
toLowerANSI _myString;            // 0.0006 ms
toLower _myString;                // 0.0016 ms

toUpperANSI _myString;            // 0.0006 ms
toUpper _myString;                // 0.0016 ms
```

## Equivalent Data Structures Performance

### Key-Value Data Structures

```sqf
private _hashMap = createHashMapFromArray [["id", 123], ["name", "player name"], ["unit", player]]; // since Arma 3 v2.02
private _goodFormat = [["id", "name", "unit"], [123, "player name", player]];
private _slowFormat = [["id", 123], ["name", "player name"], ["unit", player]];
```

```sqf
private _name = _hashMap get "name"; // 0.0018ms

// this takes 0.0038ms:
private _index = _goodFormat select 0 find "name"; // loop in engine
private _name = _goodFormat select 1 select _index;

// this takes 0.0116ms:
private _index = _slowFormat findIf { _x select 0 == "name" }; // loop in script
private _name = _slowFormat select _index select 1;
```

> **Note:** Database Functions use a slow format.

## Conversion From Earlier Versions

Each iteration of Bohemia games (OFP, ArmA, ArmA1, ArmA2, TKN, Arma 3) brought their own new commands, especially ArmA2 and Arma 3. For that, if you are converting scripts from older versions of the engine, the following aspects should be reviewed.

### Loops

- `forEach` loops, depending on the situation, can be replaced by:
  - `apply`
  - `count`
  - `findIf`
  - `select`

### Array Operations

- **Adding an item:** `myArray + [element]` and `myArray set [count myArray, element]` have been replaced by `pushBack`
- **Selecting a random item:** `BIS_fnc_selectRandom` has been replaced by `selectRandom`
- **Removing items:** `myArray set [1, objNull]; myArray - [objNull]` has been replaced by `deleteAt` and `deleteRange`
- **Concatenating:** `myArray = myArray + [element]` has been reinforced with `append`: if you don't need the original array to be modified, use `+`
- **Comparing:** use `isEqualTo` instead of `BIS_fnc_arequal`
- **Finding common items:** `in` `forEach` loop has been replaced by `arrayIntersect`
- **Condition filtering:** `forEach` can be replaced by `select` (alternative syntax)

```sqf
result = (arrayOfNumbers select { _x % 2 == 0 });    // 1.55 ms
```

```sqf
result = [];
{
    if (_x % 2 == 0) then { result pushBack _x; };
} forEach arrayOfNumbers;                            // 2.57 ms
```

### Vector Operations

- `BIS_fnc_vectorMultiply` has been replaced by `vectorMultiply` (at least 6x faster)
- `BIS_fnc_vectorDivide` too, to an extent

```sqf
private _vector = [102, 687, 1543];
private _factor = 53;

_vector vectorMultiply _factor;                    // 0.0028 ms
[_vector, _factor] call BIS_fnc_vectorMultiply;    // 0.0145 ms

_vector vectorMultiply (1 / _factor);              // 0.003 ms - but beware of 0 divisor
[_vector, _factor] call BIS_fnc_vectorDivide;      // 0.017 ms
```

### String Operations

String manipulation has been simplified with the following commands:

- Alternative syntax for `select`: `string select index`
- `toArray` and `toString` have been reinforced with `splitString` and `joinString`

### Number Operations

- `BIS_fnc_linearConversion` has been replaced by `linearConversion`. The command is **9 times faster**.
- `BIS_fnc_selectRandomWeighted` has been replaced by `selectRandomWeighted`. The command is **7 times faster**.

### Type Comparison

- `typeName` has been more than **reinforced** with `isEqualType`.

### Multiplayer

- `BIS_fnc_MP` has been replaced by `remoteExec` and `remoteExecCall` and internally uses them. Use the engine commands from now on!

### Parameters

- `BIS_fnc_param` has been replaced by `param` and `params`. The commands are approximately **14 times** faster. Use them!
