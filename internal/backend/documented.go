/*
 * Copyright 2023 Dominic Kramer
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package backend

import (
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/mlglib"
	"regexp"
	"strings"
)

type TextItemType interface {
	TextItemType()
}

func (StringItem) TextItemType()       {}
func (SubstitutionItem) TextItemType() {}

type StringItem struct {
	Text string
}

type SubstitutionItem struct {
	Name       string
	NameSuffix string
	IsVarArg   bool
	Prefix     string
	Suffix     string
	Infix      string
}

type CalledSummary struct {
	Called string
}

type WrittenSummary struct {
	Written string
}

type WritingSummary struct {
	Form    PatternType
	Written string
}

func ToCalledSummaries(node ast.CalledGroup) []CalledSummary {
	result := make([]CalledSummary, 0)
	for _, item := range node.Called.Called {
		result = append(result, CalledSummary{
			Called: item.RawText,
		})
	}
	return result
}

func ToWrittenSummaries(node ast.WrittenGroup) []WrittenSummary {
	result := make([]WrittenSummary, 0)
	for _, item := range node.Written.Written {
		result = append(result, WrittenSummary{
			Written: item.RawText,
		})
	}
	return result
}

func ToWritingSummaries(node ast.WritingGroup) []WritingSummary {
	result := make([]WritingSummary, 0)
	for _, item := range node.As.As {
		result = append(result, WritingSummary{
			Form:    ToPatternFromTarget(node.Writing.Writing),
			Written: item.RawText,
		})
	}
	return result
}

func GetWrittenSummaries(documented *ast.DocumentedSection) []WrittenSummary {
	summaries := make([]WrittenSummary, 0)
	if documented == nil {
		return summaries
	}
	for _, docItem := range documented.Documented {
		switch item := docItem.(type) {
		case *ast.WrittenGroup:
			for _, text := range item.Written.Written {
				summaries = append(summaries, WrittenSummary{
					Written: text.RawText,
				})
			}
		}
	}
	return summaries
}

func GetWritingSummaries(documented *ast.DocumentedSection) []WritingSummary {
	summaries := make([]WritingSummary, 0)
	if documented == nil {
		return summaries
	}
	for _, docItem := range documented.Documented {
		switch item := docItem.(type) {
		case *ast.WritingGroup:
			for _, as := range item.As.As {
				summaries = append(summaries, WritingSummary{
					Form:    ToPatternFromTarget(item.Writing.Writing),
					Written: as.RawText,
				})
			}
		}
	}
	return summaries
}

func GetCalledSummaries(documented *ast.DocumentedSection) []CalledSummary {
	summaries := make([]CalledSummary, 0)
	if documented == nil {
		return summaries
	}
	for _, docItem := range documented.Documented {
		switch item := docItem.(type) {
		case *ast.CalledGroup:
			for _, text := range item.Called.Called {
				summaries = append(summaries, CalledSummary{
					Called: text.RawText,
				})
			}
		}
	}
	return summaries
}

func ParseCalledWritten(text string) ([]TextItemType, error) {
	nameMatch := regexp.MustCompile(`[a-zA-Z0-9]+(\+|-|=)?\?`)
	result := make([]TextItemType, 0)
	for len(text) > 0 {
		indices := nameMatch.FindStringIndex(text)
		if indices == nil {
			break
		}
		prefix := text[0:indices[0]]

		if len(prefix) > 0 {
			result = append(result, StringItem{
				Text: prefix,
			})
		}

		// use indices[1]-1 to ignore the trailing ?
		match := text[indices[0] : indices[1]-1]
		name := ""
		nameSuffix := ""
		if strings.HasSuffix(match, "+") || strings.HasSuffix(match, "-") ||
			strings.HasSuffix(match, "=") {
			name = match[0 : len(match)-1]
			nameSuffix = match[len(match)-1:]
		} else {
			name = match
		}

		suffix := text[indices[1]:]

		text = suffix

		inner := ""
		if len(text) > 0 && text[0] == '{' {
			inner += "{"
			stack := mlglib.NewStack[byte]()
			stack.Push('{')
			i := 1
			for i < len(text) {
				c := text[i]
				i += 1
				if c == '{' {
					stack.Push(c)
				} else if c == '}' {
					if !stack.IsEmpty() && stack.Peek() == '{' {
						stack.Pop()
					}
					if stack.IsEmpty() {
						inner += string(c)
						break
					}
				}
				inner += string(c)
			}
			text = text[len(inner):]

			// inner is of the form {...}, so remove the leading { and trailing }
			inner = inner[1 : len(inner)-1]

			innerPrefix := ""
			hasDotDotDot1 := false
			innerInfix := ""
			hasDotDotDot2 := false
			innerSuffix := ""

			j := 0
			for j < len(inner) && inner[j] != '.' {
				innerPrefix += string(inner[j])
				j += 1
			}

			if j+2 < len(inner) && inner[j] == '.' && inner[j+1] == '.' && inner[j+2] == '.' {
				hasDotDotDot1 = true
				j += 3
			}

			for j < len(inner) && inner[j] != '.' {
				innerInfix += string(inner[j])
				j += 1
			}

			if j+2 < len(inner) && inner[j] == '.' && inner[j+1] == '.' && inner[j+2] == '.' {
				hasDotDotDot2 = true
				j += 3
			}

			for j < len(inner) {
				innerSuffix += string(inner[j])
				j += 1
			}

			if !hasDotDotDot1 && !hasDotDotDot2 {
				return nil, fmt.Errorf("At least one .... expected")
			}

			result = append(result, SubstitutionItem{
				Name:       name,
				NameSuffix: nameSuffix,
				IsVarArg:   true,
				Prefix:     innerPrefix,
				Infix:      innerInfix,
				Suffix:     innerSuffix,
			})
		} else {
			result = append(result, SubstitutionItem{
				Name:       name,
				NameSuffix: nameSuffix,
				IsVarArg:   false,
			})
		}
	}
	return result, nil
}
