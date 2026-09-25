---
title: "Scripting Overview"
category: "scripting-tutorial"
source: "scriptingpageswiki.txt"
description: "Learning roadmap: beginner topics (variables, commands, control structures, arrays, script files), intermediate (functions, namespaces, event handlers, hashmaps), advanced (multiplayer, GUIs, optimisation, debugging)."
---

# Scripting Overview

## Scripting Topics

There is a plethora of topics to learn about in the context of ArmA scripting. Fortunately, there is no need to acquire detailed knowledge about all of these things in order to get started with scripting. To aid with prioritisation, this page provides three selections of topics, each relevant to beginner, intermediate and advanced scripters respectively.

### Beginner Topics

- Variables
- Commands and Operators
- Control Structures, Conditions and Booleans
- Arrays
- Script Files

### Intermediate Topics

- Functions
- Namespaces and Scopes
- Event Handlers
- HashMaps

### Advanced Topics

- Multiplayer Scripting
- GUIs
- Code Optimisation
- Debugging Techniques

## Beginner Scripting

### Variables

Variables are placeholders for values. They always have a **Data Type** (Number, String, Boolean, Array, etc.) that changes automatically based on the assigned value.

```sqf
A = 1;           // Number
B = "Hello";     // String
C = [1,2,3];     // Array
```

### Commands and Operators

**Operators** (`+`, `-`, `*`, `/`, `=`, `&&`, `||`, `!`) perform basic operations. **Commands** interact with the game (e.g., `setPosATL`, `damage`, `systemChat`).

```sqf
A = 1.5;
B = -2 * A;
C = A + B + 3.5; // Result: C is 2
```

Every command has a Wiki page with its behaviour, return value, and parameter types.

> **Note:** Full list: [Scripting Commands](https://community.bistudio.com/wiki/Category:Scripting_Commands).

### Control Structures, Conditions and Booleans

**Control Structures** enable branching and loops:

```sqf
if (damage player > 0.5) then
{
    player setDamage 0;
    systemChat "The player has been healed.";
}
else
{
    systemChat "The player is fine.";
};
```

This code behaves differently depending on the damage status of the player. The `systemChat` output and whether or not the player is healed changes dynamically based on how much health the player has when the code is executed.

#### Conditions

In the example above, the condition is `damage player > 0.5`. Like all conditions, it results in a Boolean value when evaluated:

1. First, `damage player` is evaluated and returns a number.
2. Then, the `>` operator compares that number to 0.5:
   - If the number is greater than 0.5, the `>` operator returns `true`.
   - If the number is less than or equal to 0.5, the `>` operator returns `false`.

#### Booleans

The data type Boolean only has two possible values: `true` and `false`.

##### Boolean Operations

There are three basic operations that can be performed on Boolean values:

| Operation | Description | SQF Operator | SQF Command |
|-----------|-------------|:---:|:---:|
| NOT (Negation) | Inverts the input value. | `!` | `not` |
| AND (Conjunction) | Combines two Booleans into one. Only returns `true` if both input values are `true`. | `&&` | `and` |
| OR (Disjunction) | Combines two Booleans into one. Returns `true` if at least one of the input values is `true`. | `\|\|` | `or` |

Both the input and the output of these operations are Boolean values. Their behaviour is defined as follows:

**NOT**

| Expression | Result |
|------------|--------|
| `!true` | `false` |
| `!false` | `true` |

**AND**

| Expression | Result |
|------------|--------|
| `true && true` | `true` |
| `true && false` | `false` |
| `false && true` | `false` |
| `false && false` | `false` |

**OR**

| Expression | Result |
|------------|--------|
| `true \|\| true` | `true` |
| `true \|\| false` | `true` |
| `false \|\| true` | `true` |
| `false \|\| false` | `false` |

#### Complex Conditions

Boolean operations can be used to create complex conditions by combining multiple conditions into one.

```sqf
if ((alive VIP_1 && triggerActivated VIP_1_Task_Complete) || (alive VIP_2 && triggerActivated VIP_2_Task_Complete)) then
{
    systemChat "At least one VIP has been rescued.";
};
```

> **Note:** Beginners sometimes write conditions such as `if (Condition == true)` or `if (Condition == false)`. While doing so is not a real error, it is unnecessary because Control Structures (and Triggers) always implicitly compare the condition to `true`. The correct (as in faster, more common and more readable) way to write such conditions is `if (Condition)` and `if (!Condition)` respectively.

### Arrays

Arrays hold ordered lists of values (any mix of types):

```sqf
[]                                        // empty
[true]                                    // [Boolean]
[0, 10, 20, 30, 40, 50]                   // [Number, ...]
["John Doe", 32, true]                    // mixed types
```

#### Positions

Positions are 2D/3D arrays: `[X, Y]` or `[X, Y, Z]` (West-East, South-North, altitude).

> **Note:** `[0, 0, 0]` ASL (**A**bove **S**ea **L**evel) and `[0, 0, 0]` AGL (**A**bove **G**round **L**evel) are not equivalent.

### Script Files

Scripts use `.sqf` (or `.sqs`); config files use `.ext`, `.hpp`, `.cpp`, `.cfg`.

#### File Creation

Enable "File name extensions" in Windows File Explorer View tab to avoid creating `file.txt` instead of `file.sqf`. Bypass Notepad by quoting the filename: `"description.ext"`.

#### File Locations

Script files go in the **scenario folder** (next to `mission.sqm`). `Description.ext` and Event Scripts go in the root; everything else can be in subfolders:

```
MyMission.Tanoa/
├── functions/
│   ├── fn_myFirstFunction.sqf
├── scripts/
│   ├── myFirstScript.sqf
├── description.ext
├── initPlayerLocal.sqf
├── initServer.sqf
└── mission.sqm
```

Open the scenario folder from Eden Editor: **Scenario → Open Scenario Folder**.

#### Editing Script Files

Use a proper editor with **syntax highlighting** (VS Code, Notepad++). Notepad works but is impractical.

#### Script Execution

```sqf
execVM "scripts\myFirstScript.sqf";
```

Can be run from: other scripts, Debug Console, Trigger On Activation/Deactivation, or object init fields.

## Intermediate Scripting

- [Functions](https://community.bistudio.com/wiki/Category:Functions)
- Namespaces, Variables#Scopes (Scopes)
- Event Handlers
- HashMaps

## Advanced Scripting

- Multiplayer Scripting
- GUIs
- Code Optimisation
- Debugging Techniques

## Miscellaneous

### Before anything

**Is your idea necessary?**
: Will players even notice or use what you want to script? Just because you can does not mean you should. Sometimes less is more!

**Is it possible to do this in the editor?**
: The Eden Editor is a powerful tool and with it alone one can achieve a lot of things without writing a single line of code. Poorly written scripts are a frequent cause of poor performance, both in singleplayer and multiplayer scenarios.

**Can it be scripted using SQF?**
: This question might be hard to answer. Try to get as much information about what you want to do and what commands and functions there are before spending time on writing a script, just to find out that what you hoped to achieve is not possible after all.

> **Note:** Scripting is **not** the solution for everything!

### Terms

The following is a collection of terms frequently encountered when talking or reading about scripting.

**Game Engine**
: The core program of the game which executes your scripting commands at run time.

**Script / Script File**
: Scripts are usually placed in script files. Script files contain code.

**Syntax**
: See SQF Syntax (ArmA, ArmA2, Arma 3). See SQS Syntax (OFP, ArmA).

**Variables**
: A Variable is a named storage container for data. The name of a variable is called its Identifier.

**Data Types**
: The Data Type of a variable specifies which kind of data that variable can contain.

**Operators**
: See Operators

**Control Structures**
: See Control Structures

**Functions**
: See Function

