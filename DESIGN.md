# Design Document

## Introduction and Overview

This document outlines the design of a Wordle solver program that allows user to determine current day of the word efficiently by using bank of official words. The goal is to create a lightweight static single page application with intuitive minimalistic interface that will solve Wordle.

### Problem Statement

Current implementations don't provide user with immediate response about possible solutions. Also they include a lot of ads/trackers and even application itself weights less than 500kb total page size is around 5mb or more.

### Scope

#### In-Scope
 - Real-time solution output
 - Minimalistic interface
 - Final size optimizations
 - Adaptability for mobile platform (small screen sizes)

#### Out of Scope
 - Multilingual support
 - Application state storage (saving of previously inputted information)

### Tenets: The Guiding Principles

This are non-negotiable points for this system design:
 - Performance. Average solution time must be less than 1ms for average user.
 - Size. The full page with code and styling should be less than 200kb in packaged form.
 - Simplicity. Application should not contain any visual clutter or unnecessary code.
 - Rust-only. All of application code must be written in Rust (except, *unfortunately*, WASM loader itself).

### Risks

Risk: WASM bundle size may exceed 200kb target
 - While it is possible to try to optimise size of WASM bundle during the feature development. It is counterproductive. The bundle size optimization will be the last on the development cycle.

Risk: Word bank accuracy/legality (2039 words source)
 - It is out of scope for this project.

Risk: Browser compatibility for WASM
 - As WASM support is widely available we will not mitigate this. However appropriate message for user without Javascript/WASM support must be displayed.

Risk: Mobile performance on low-end devices
 - Performance on low-end devices is not taken into account for performance goal, but we should strive for comparable numbers.

Risk: Creating a single HTML file page may be not feasible due to WASM code being inflated while converted string via base64 or base122.
 - Mitigation: split HTML and WASM code to 2 different files and link them together.

### Assumptions
 - Users will primarily access application through desktop and mobile browsers.
 - All computations will be run on the client side.
 - Users use official Wordle game implementation with 2039 words dictionary.

## System Architecture

The application moving parts consist of two big pieces: game solver algorithm and on-DOM visualization logic.
As well as HTML page with styles.

```mermaid
architecture-beta
    group app[Web Page]

    service user(mdi:account)[User]

    service HTML(mdi:code-block-html)[HTML] in app
    service visualizer(mdi:filter-gear)[Visualizer] in app
    service solver(mdi:gear)[Game Solver] in app

    solver:R -- L:visualizer
    visualizer:R -- L:HTML
    HTML:R <-- L:user
```

### Game Solver

Game solver provides only one single API point named `solve` which takes as arguments:
 - `correct_letters` - 5-lettered strings with guessed so far letters, not yet guessed letters are presented as space.
 - `misplaced_letters` - list of 5-lettered strings which contain misplaced letters and their exact guess position. Array length is not capped.
 - `excluded_letters` - a string that contains all of letters that are guessed incorrectly by user.

All of arguments are case-insensitive.

Game solver should not have any internal state as any caching logic is outside of its responsibilities.

### Visualization Logic

This module is responsible for user input handling, managing state of page and rendering game solver results for user.

> *Important note:*
> The visualizer manages DOM state only while solver is stateless.

As one of goals is simplicity, state of application will be fully determined by user input on the page.
This of course will inevitably increase computation requirements of application though it is deemed reasonable.

Caching of previous results are acceptable and left for a developer to decide.

## Data Design

The main decision is representing letters that are contained in a word by using unsigned 32-bit integer as we need only 26 bits to fully represent English alphabet.
It used as a mask should allow for easier comparison to letters that are in the word (correct and misplaced) and excluded letters in only 2 bitwise operations which will (presumably) speed up initial search for candidates.

All words will be stored in single constant byte-array. Padding of said array is optional and should be decided after careful testing in performance difference.
That said array without padding is preferable due to raw size of data 9.96kb without padding vs 15.92kb with padding.

### User Input Validation

Any non-ASCII letter character in user input should be treated as error and block any additional data manipulation.

Even though game solving logic is expected to receive sanitised user input it must recognize any invalid input and return to user code error value with the reason.

### Word Bank

#### Source
- User-provided word lists in data/ folder
- MVP uses solutions.txt only (2,309 words)

#### Storage Format
- No spaces, continuous 5-byte chunks
- Total size: 11,545 bytes (2,309 × 5)

```rust
// words.rs - generated from data/solutions.txt
pub const WORD_COUNT: usize = 2309;
pub const WORDS: &[u8; 11545] = b"abackabaseabate...zinch";
// Access word at index i: &WORDS[i*5..(i+1)*5]
```

#### Pre-computed Masks (Optional Optimization)
- Letter bitmask for each word stored as parallel array
- Allows O(1) letter existence check

## Component Design

There are three major components in the application.

### HTML Page

#### Purpose and Responsibilities
It's responsible for presenting information to the user and receiving inputs from the user.

#### Input and Output Specifications
The direct input for the page consists of user interactions and algorithm results that are propagated through the Visualizer.

Outputs are the user inputted data and events that this data was updated.

#### Algorithms and Processing Logic
There is none.

#### Dependencies on Other Components or External Systems
As page doesn't contain any logic, It's dependent on visualization component to handle user input validation/sanitisation and rendering results on the page.

### Visualizer

#### Purpose and Responsibilities
It's responsible for processing user inputs, triggering solver logic and rendering results onto the page.

#### Input and Output Specifications
The inputs are data from page inputs fields such as current correctly guessed letters, misplaced letters and excluded letters.

Outputs are list of the guesses outputted by the solver that visualizer formats and renders onto the page.

#### Algorithms and Processing Logic

There are no major algorithms involved. Only checks for data correctness: expected format, boundary checks.

If any error detected in user inputs, sanitise incorrect data and replace the original. If error occurred in the solver or too critical to continue processing, show error and stop processing. Processing should restart when inputs are corrected.

#### Dependencies on Other Components or External Systems

This component is dependent on the page to receive inputs and on the solver to process correctly formatted data.

### Game Solver

#### Purpose and Responsibilities
The heart of the application. It contains algorithm to sort through word bank and return possible valid guesses.

#### Input and Output Specifications
As discussed earlier in this document, the inputs are correctly formatted and sanitised strings: guessed letters, misplaced letters and excluded letters.

#### Algorithms and Processing Logic
As baseline algorithm brute-force filtering should be used and guessed must be ranked according most common not yet guessed letters in them.

When system matures we expect to test a few different algorithms for faster filtering and ranking the results.

#### Dependencies on Other Components or External Systems
As far as we know there is no direct dependencies on other components.

## Error Handling

#### Error Types
```rust
pub enum SolverError {
    InvalidCharacter(char),   // Non-ASCII character found
    InvalidLength(usize),     // String not 5 characters
    EmptyInputs,              // ALL of inputs are empty.
}
```

#### Error Response Format
```json
{
  "ok": false,
  "error": {
    "code": "INVALID_CHARACTER",
    "message": "Found non-ASCII character: 'ñ'"
  }
}
```

#### User-Facing Errors
- Display error message below relevant input field
- Highlight invalid input field
- Allow immediate correction without page reset

## Testing Strategy

#### Test Framework
- Standard `#[test]` for unit tests
- wasm-bindgen-test for WASM integration tests (feature-gated)

#### Test Categories

**Unit Tests - Game Solver**
- Filter matches letters at exact positions
- Filter excludes words with excluded letters
- Filter handles misplaced letters correctly
- Edge cases: no matches returns empty, all letters guessed
- Ranking sorts by letter frequency

**Unit Tests - Input Validation**
- Rejects non-ASCII characters
- Rejects strings != 5 characters
- Handles case insensitivity
- Empty optional fields handled

**Integration Tests**
- solve() returns correct JSON structure
- Error cases return proper error codes

#### Test Data
- Subset of solution words for fast testing
- Known Wordle answers for verification

## Build & Deployment

#### Toolchain
- Rust stable toolchain
- trunk-rs for building and dev server
- wasm-bindgen for WASM bindings (internal)

#### Build Commands
```bash
trunk serve        # Development with hot-reload
trunk build --release  # Production build
```

#### Output Structure
```
dist/
├── index.html
├── style.css
├── wordle_finder-[hash].wasm
└── wordle_finder-[hash].js
```

#### JS Bindings
<!-- TODO: Update if needed -->
- Internal only, not exposed as library
- Could be feature-gated in Cargo.toml if needed later:
```toml
[features]
default = []
js-bindings = []
```

#### Deployment
- Automatic using GitHub Actions
- GitHub Pages from dist/ folder
- No server-side logic required

## File Structure

```
wordle-finder/
├── src/
│   ├── main.rs              # WASM entry point
│   ├── solver/
│   │   ├── mod.rs          # Public API (solve function)
│   │   ├── validator.rs    # Input validation
│   │   ├── filter.rs       # Word filtering logic
│   │   └── rank.rs         # Result ranking
│   ├── visualizer.rs       # Visualizer module (DOM logic)
│   └── words.rs            # Word bank constant
├── data/
│   ├── solutions.txt       # 2,309 solution words (user-provided)
│   └── guesses.txt         # Valid guess words (for future use)
├── static/
│   ├── index.html
│   └── style.css
├── tests/
│   └── solver_tests.rs     # Integration tests
├── Cargo.toml
└── DESIGN.md
```

## User Interface

Style of the page must be minimalistic close to original game style. Site must use 5 colours colour palette that may or may not be changed during development.

Most input fields are presented as block of 5 letters for each "word".
1 such word for correctly guessed letters, and 1 or more words for misplaced letters. Excluded letters must be presented as single text input field.

In the desktop version inputs are located on left side of the screen and place for outputs will be located on the right side.

In the mobile version layout is vertical where inputs are above the outputs. In mobile version taking full width minus the padding is expected.

Button for clearing all of inputs are expected.

#### Misplaced Letters Input

As there are 1 or more input fields for misplaced letters, there must be mechanism to add/remove inputs. Maximum number of input "words" is 5 (max guesses in Wordle - 1). We propose a small button to the right of each "word" that will delete this "word" and a big button at the bottom of the column that will add new "word" input.

Deletion logic is simple "delete current 'word' input if there more than 1 input left, otherwise clear it".
