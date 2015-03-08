// The MIT License (MIT)
//
// Copyright (c) 2015 TAKAHASHI Satoshi <hikobae@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
package main

import (
	"flag"
	"fmt"
	"os"
	"log"
	"path/filepath"
	"strings"
)

const allFlagDescription = "Print all pathnames of each matching filename."

var allFlag = flag.Bool("all", false, allFlagDescription)

func usage() {
	fmt.Fprintf(os.Stderr, `Usage: which [options] filename [...]

which print the pathname of the filename in the current environment.

Options:
`)
	flag.PrintDefaults()
	os.Exit(2)
}

func exists(filename string) bool {
	_, err := os.Stat(filename)
	return !os.IsNotExist(err)
}

func printIfExists(filepath string) bool {
	pathext := strings.ToLower(os.Getenv("PATHEXT"))

	exts := []string{""}
	exts = append(exts, strings.Split(pathext, ";")...)

	existed := false
	for _, ext := range exts {
		path := filepath + ext
		if !exists(path) {
			continue
		}
		fmt.Println(path)
		existed = true
		if !*allFlag {
			break
		}
	}
	return existed
}

func which(filename string, paths []string) bool {
	existed := false
	for _, d := range paths {
		if !printIfExists(filepath.Join(d, filename)) {
			continue
		}
		existed = true
		if !*allFlag {
			break
		}
	}
	return existed
}

func getPaths() []string {
	wd, err := os.Getwd()
	if err != nil {
		log.Fatal(err)
	}

	paths := []string{wd}

	p := strings.Split(os.Getenv("PATH"), string(filepath.ListSeparator))
	paths = append(paths, p...)
	return paths
}

func init() {
	flag.BoolVar(allFlag, "a", false, allFlagDescription)
}

func main() {
	flag.Usage = usage
	flag.Parse()

	paths := getPaths()

	fail := false
	for _, f := range flag.Args() {
		if !which(f, paths) {
			fail = true
		}
	}

	if fail {
		os.Exit(1)
	}
}
