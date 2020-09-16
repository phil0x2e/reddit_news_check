![Release version](https://img.shields.io/github/v/release/phil0x2e/reddit_news_check)

# Reddit News Checker
Checks if there are new posts on subreddits or users.

Just specify the urls to the profiles or subreddits in a line sperated csv file and specify the number of days, in which to check for new posts.
0 days means anything under a day, so minutes or hours before. 1 day means 1 day and less ago etc.

Run with --help to get help.

Important:
 - urls have to start with https://www.
 - At this point only works with days, so when specifying days greater than 30 it won't work as expected, because reddit starts with months after that.
