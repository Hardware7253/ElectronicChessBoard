# Electronic Chess Board
Welcome to the Electronic Chess Board project repository. This project combines an electronic chess board with a chess engine, offering a more tactile way to train against an AI opponent.

## Overview
The project's core objective is to deliver an interactive chess experience, allowing users to engage in matches against an AI opponent.
This AI opponent employs a minimax algorithm for strategic decision-making, with a variable search depth of 3 to 6 based on time availability. 
The chess board integrates 64 hall effect sensors to precisely detect piece positions, while the AI's moves are visually indicated using 64 LEDs.

## Code
In the [Code](/Code) directory, you'll find the firmware that powers the project. This firmware includes the chess engine responsible for AI decision-making.
The project is programmed in embedded rust, and is designed to run on the custom STM32F103C8T6 chess board.

## CAD
The [CAD](/Cad) directory contains KiCad schematics for the custom chess board PCB. Additionally, you'll find laser cutting files and 3D design files for the casing of the electronic chess board.
Manufacturing outputs for the PCB can be found in the [Manufacture](/Cad/ChessBoardKiCad/Manufacture) directory.

# Manual

## Starting the game
To begin playing, the game board will prompt you to select your team—black or white. Simply press the button while your desired team is displayed on the screen.

## Setting up the board
After starting the game, you'll be prompted to set up the chessboard. Arrange the 32 chess pieces in their starting positions, with the color that represents your team closest to you. For instance, if you are playing as the black team, ensure that the black pieces are positioned on the bottom of the board closest to you. If you need assistance during this process, you can hold down the button at any time. Doing so will illuminate LEDs on the board, indicating missing pieces or incorrectly placed ones. Once all pieces are correctly positioned, the game will automatically commence.

## Playing the game
The game follows a turn-based structure with both the user and the chess engine taking their respective turns. To make a move, simply move a chess piece and press the button to confirm that you have completed your turn. When it's the computer's turn to move, it will signal its move by blinking LEDs to indicate the destination square.

## Capturing
When capturing an opponent's piece, start by removing the captured piece from the board. Then, move your piece to its final destination and press the button to signal the end of your turn. This behavior is consistent with en passant moves, where you must first remove the captured pawn from the board before completing your move.

## Castling
To execute a castling move, move your king to its castled position without moving the rook. Press the button to indicate that you have finished your turn. After this, you may move your rook to the opposite side of the king to complete the castling maneuver.

## Errors
Errors are most likely to occur when an illegal move is made. The board's LEDs will illuminate to highlight the pieces that need adjustment to rectify the error. Additionally, the LCD screen will prompt you to revert the piece positions.

Errors may also happen when pieces are moved during the chess engine's turn. In such cases, you may notice multiple LEDs being lit up, some indicating the engine's move and others indicating pieces in the wrong position. This typically occurs when one or more pieces are slightly off-center on their squares. To resolve this issue, simply adjust the misaligned pieces inward slightly to correct the error.
