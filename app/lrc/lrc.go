package lrc

import (
	"bufio"
	"html"
	"io"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"
)

var (
	timedLineRegexp    = regexp.MustCompile(`^((?:\[\d{2,}:\d{2,}\.\d{2,}\])+)(.*)`)
	timeSegmentsRegexp = regexp.MustCompile(`\[(\d{2,}):(\d{2,})\.(\d{2,})\]`)
)

type Lyrics struct {
	lines     []LyricsLine
	lastIndex int
}

func Parse(r io.Reader) (*Lyrics, error) {
	lyrics := new(Lyrics)
	if err := lyrics.parse(r); err != nil {
		return nil, err
	}
	return lyrics, nil
}

func (l *Lyrics) Line(timeTag time.Duration) LyricsLine {
	n := len(l.lines)
	if l.lastIndex < n {
		// Fast-path.
		// TODO: Interval tree?

		var (
			low, high LyricsLine
		)
		low = l.lines[l.lastIndex]
		if l.lastIndex+1 < n {
			high = l.lines[l.lastIndex+1]
		} else {
			high = l.lines[n-1]
		}
		if low.TimeTag <= timeTag && timeTag < high.TimeTag {
			return l.lines[l.lastIndex]
		}
	}
	i := sort.Search(n, func(i int) bool { return l.lines[i].TimeTag >= timeTag })
	if i == 0 {
		l.lastIndex = 0
		return LyricsLine{}
	}
	l.lastIndex = i - 1
	return l.lines[i-1]
}

func (l *Lyrics) parse(r io.Reader) error {
	scanner := bufio.NewScanner(r)
	scanner.Split(bufio.ScanLines)
	for scanner.Scan() {
		l.parseLine(scanner.Text())
	}
	sort.Slice(l.lines, func(i, j int) bool { return l.lines[i].TimeTag < l.lines[j].TimeTag })
	return nil
}

func (l *Lyrics) parseLine(line string) {
	matches := timedLineRegexp.FindStringSubmatch(line)
	if len(matches) != 3 {
		return
	}
	timeTagStrings := matches[1]
	text := strings.TrimSpace(html.UnescapeString(matches[2]))
	for _, timeTag := range l.parseTimeTagStrings(timeTagStrings) {
		l.addParsedLine(timeTag, text)
	}
}

func (l *Lyrics) parseTimeTagStrings(s string) []time.Duration {
	var timeTags []time.Duration
	for _, timeTagString := range timeSegmentsRegexp.FindAllStringSubmatch(s, -1) {
		minutes, err := strconv.Atoi(timeTagString[1])
		if err != nil {
			continue
		}
		seconds, err := strconv.Atoi(timeTagString[2])
		if err != nil {
			continue
		}
		hundredthsOfSeconds, err := strconv.Atoi(timeTagString[3][:2])
		if err != nil {
			continue
		}
		timeTag := time.Duration(minutes)*time.Minute + time.Duration(seconds)*time.Second + time.Duration(hundredthsOfSeconds*10)*time.Millisecond
		timeTags = append(timeTags, timeTag)
	}
	return timeTags
}

func (l *Lyrics) addParsedLine(timeTag time.Duration, text string) {
	l.lines = append(l.lines, LyricsLine{TimeTag: timeTag, Text: text})
}

type LyricsLine struct {
	TimeTag time.Duration
	Text    string
}
