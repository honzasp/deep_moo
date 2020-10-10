# Deep moo

Would you like to defeat your friends in "6 nimmt" and completely miss the point
of this party game at the same time? Then this program is for you!

You have to provide a text file with description of the game as the first (and
only) command line argument. The format of this file is as follows:

    # This line specifies the cards in your hand.
    h 10 20 30 40 50 60 70 80 90 100
    # This line gives the names of the players. The first player is always
    # assumed to be me (the player controlled by the program).
    p me alice bob

    # For every round of the game that has already been played, this section
    # specifies the cards on the table...
    t 2 4 10
    t 48 49 68 84
    t 51 64 72
    t 9 22 32 47 50
    # ...and the actions that players (including me) played. The cards are
    # always given in the same order as in the 'p' line
    a 20 61 80

    # This specifies the current situation on the table
    t 2 4 10 20
    t 48 49 68 84
    t 51 64 72 80
    t 61

The output of the program looks as follows:

     30   6.36
     10   4.16
     40   4.09
     50   2.88
     60   2.27
     90   1.13
    100   1.12
     70   0.58
     80   0.57

This orders the cards in hand from the best to the worst and prints the expected
advantage of playing each card.
