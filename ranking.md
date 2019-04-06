# Trending / Hot / Best Sorting algorithm
## Goals
- During the day, new posts and comments should be near the top, so they can be voted on.
- After a day or so, the time factor should go away.
- Use a log scale, since votes tend to snowball, and so the first 10 votes are just as important as the next hundred.

## Reddit Sorting
[Reddit's comment sorting algorithm](https://medium.com/hacking-and-gonzo/how-reddit-ranking-algorithms-work-ef111e33d0d9), the wilson confidence sort, is inadequate, because it completely ignores time. What ends up happening, especially in smaller subreddits, is that the early comments end up getting upvoted, and newer comments stay at the bottom, never to be seen.

## Hacker News Sorting
The [Hacker New's ranking algorithm](https://medium.com/hacking-and-gonzo/how-hacker-news-ranking-algorithm-works-1d9b0cf2c08d) is great, but it doesn't use a log scale for the scores.

## My Algorithm
```
Rank = ScaleFactor * sign(Score) * log(1 + abs(Score)) / (Time + 2)^Gravity

Score = Upvotes - Downvotes
Time = time since submission (in hours)
Gravity = Decay gravity, 1.8 is default
```

- Add 1 to the score, so that the standard new comment score of +1 will be affected by time decay. Otherwise all new comments would stay at zero, near the bottom.
- The sign and abs of the score are necessary for dealing with the log of negative scores.
- A scale factor of 10k gets the rank in integer form.

A plot of rank over 24 hours, of scores of 1, 5, 10, 100, 1000, with a scale factor of 10k.

![](https://i.imgur.com/w8oBLlL.png)
