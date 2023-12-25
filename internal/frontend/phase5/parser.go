/*
 * Copyright 2022 Dominic Kramer
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

package phase5

import (
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/formulation"
	"mathlingua/internal/frontend/phase4"
	"mathlingua/internal/mlglib"
)

func Parse(
	doc phase4.Document,
	path ast.Path,
	tracker frontend.IDiagnosticTracker,
	keyGen mlglib.IKeyGenerator,
) (ast.Document, bool) {
	p := parser{
		path:    path,
		tracker: tracker,
		keyGen:  keyGen,
	}
	return p.toDocument(doc)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type parser struct {
	path    ast.Path
	tracker frontend.IDiagnosticTracker
	keyGen  mlglib.IKeyGenerator
}

///////////////////////////////////////// let ////////////////////////////////////////////////////

func (p *parser) toLetGroup(group phase4.Group) (ast.LetGroup, bool) {
	if !startsWithSections(group, ast.LowerLetName) {
		return ast.LetGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.LetSections...)
	if !ok {
		return ast.LetGroup{}, false
	}
	let := *p.toLetSection(sections[ast.LowerLetName])
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sect, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sect)
	}
	var suchThat *ast.SuchThatSection
	if sect, ok := sections[ast.LowerSuchThatName]; ok {
		suchThat = p.toSuchThatSection(sect)
	}
	then := *p.toThenSection(sections[ast.LowerThenName])
	return ast.LetGroup{
		Label:          label,
		Let:            let,
		Using:          using,
		Where:          where,
		SuchThat:       suchThat,
		Then:           then,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toLetSection(section phase4.Section) *ast.LetSection {
	return &ast.LetSection{
		Let:            p.oneOrMoreTargets(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

//////////////////////////////////// equivalently //////////////////////////////////////////////////

func (p *parser) toEquivalentlyGroup(group phase4.Group) (ast.EquivalentlyGroup, bool) {
	if !startsWithSections(group, ast.LowerEquivalentlyName) {
		return ast.EquivalentlyGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.EquivalentlySections...)
	if !ok {
		return ast.EquivalentlyGroup{}, false
	}
	return ast.EquivalentlyGroup{
		Label:          label,
		Equivalently:   *p.toEquivalentlySection(sections[ast.LowerEquivalentlyName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toEquivalentlySection(section phase4.Section) *ast.EquivalentlySection {
	return &ast.EquivalentlySection{
		Clauses:        p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////// allOf ////////////////////////////////////////////////////

func (p *parser) toAllOfGroup(group phase4.Group) (ast.AllOfGroup, bool) {
	if !startsWithSections(group, ast.LowerAllOfName) {
		return ast.AllOfGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.AllOfSections...)
	if !ok {
		return ast.AllOfGroup{}, false
	}
	return ast.AllOfGroup{
		Label:          label,
		AllOf:          *p.toAllOfSection(sections[ast.LowerAllOfName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toAllOfSection(section phase4.Section) *ast.AllOfSection {
	return &ast.AllOfSection{
		Clauses:        p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////// not //////////////////////////////////////////////////////

func (p *parser) toNotGroup(group phase4.Group) (ast.NotGroup, bool) {
	if !startsWithSections(group, ast.LowerNotName) {
		return ast.NotGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.NotSections...)
	if !ok {
		return ast.NotGroup{}, false
	}
	return ast.NotGroup{
		Label:          label,
		Not:            *p.toNotSection(sections[ast.LowerNotName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toNotSection(section phase4.Section) *ast.NotSection {
	return &ast.NotSection{
		Clause:         p.exactlyOneClause(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////// anyOf ///////////////////////////////////////////////////

func (p *parser) toAnyOfGroup(group phase4.Group) (ast.AnyOfGroup, bool) {
	if !startsWithSections(group, ast.LowerAnyOfName) {
		return ast.AnyOfGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.AnyOfSections...)
	if !ok {
		return ast.AnyOfGroup{}, false
	}
	return ast.AnyOfGroup{
		Label:          label,
		AnyOf:          *p.toAnyOfSection(sections[ast.LowerAnyOfName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toAnyOfSection(section phase4.Section) *ast.AnyOfSection {
	return &ast.AnyOfSection{
		Clauses:        p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////////// oneOf ////////////////////////////////////////////////

func (p *parser) toOneOfGroup(group phase4.Group) (ast.OneOfGroup, bool) {
	if !startsWithSections(group, ast.LowerOneOfName) {
		return ast.OneOfGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.OneOfSections...)
	if !ok {
		return ast.OneOfGroup{}, false
	}
	return ast.OneOfGroup{
		Label:          label,
		OneOf:          *p.toOneOfSection(sections[ast.LowerOneOfName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toOneOfSection(section phase4.Section) *ast.OneOfSection {
	return &ast.OneOfSection{
		Clauses:        p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////// exists ///////////////////////////////////////////////////////

func (p *parser) toExistsGroup(group phase4.Group) (ast.ExistsGroup, bool) {
	if !startsWithSections(group, ast.LowerExistsName) {
		return ast.ExistsGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ExistsSections...)
	if !ok {
		return ast.ExistsGroup{}, false
	}
	exists := *p.toExistsSection(sections[ast.LowerExistsName])
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sect, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sect)
	}
	var suchThat *ast.SuchThatSection
	if sect, ok := sections[ast.LowerSuchThatName]; ok {
		suchThat = p.toSuchThatSection(sect)
	}
	return ast.ExistsGroup{
		Label:          label,
		Exists:         exists,
		Using:          using,
		Where:          where,
		SuchThat:       suchThat,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toExistsSection(section phase4.Section) *ast.ExistsSection {
	return &ast.ExistsSection{
		Targets:        p.oneOrMoreTargets(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toWhereSection(section phase4.Section) *ast.WhereSection {
	return &ast.WhereSection{
		Specs:          p.oneOrMoreSpecs(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toSuchThatSection(section phase4.Section) *ast.SuchThatSection {
	return &ast.SuchThatSection{
		Clauses:        p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

/////////////////////////////////////// existsUnique ///////////////////////////////////////////////

func (p *parser) toExistsUniqueGroup(group phase4.Group) (ast.ExistsUniqueGroup, bool) {
	if !startsWithSections(group, ast.LowerExistsUniqueName) {
		return ast.ExistsUniqueGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ExistsUniqueSections...)
	if !ok {
		return ast.ExistsUniqueGroup{}, false
	}
	existsUnique := *p.toExistsUniqueSection(sections[ast.LowerExistsUniqueName])
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sect, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sect)
	}
	suchThat := *p.toSuchThatSection(sections[ast.LowerSuchThatName])
	return ast.ExistsUniqueGroup{
		Label:          label,
		ExistsUnique:   existsUnique,
		Using:          using,
		Where:          where,
		SuchThat:       suchThat,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toExistsUniqueSection(section phase4.Section) *ast.ExistsUniqueSection {
	return &ast.ExistsUniqueSection{
		Targets:        p.oneOrMoreTargets(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////// forAll ///////////////////////////////////////////////////

func (p *parser) toForAllGroup(group phase4.Group) (ast.ForAllGroup, bool) {
	if !startsWithSections(group, ast.LowerForAllName) {
		return ast.ForAllGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ForAllSections...)
	if !ok {
		return ast.ForAllGroup{}, false
	}
	forAll := *p.toForAllSection(sections[ast.LowerForAllName])
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sec, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sec)
	}
	var suchThat *ast.SuchThatSection
	if sec, ok := sections[ast.LowerSuchThatName]; ok {
		suchThat = p.toSuchThatSection(sec)
	}
	then := *p.toThenSection(sections[ast.LowerThenName])
	return ast.ForAllGroup{
		Label:          label,
		ForAll:         forAll,
		Using:          using,
		Where:          where,
		SuchThat:       suchThat,
		Then:           then,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toForAllSection(section phase4.Section) *ast.ForAllSection {
	return &ast.ForAllSection{
		Targets:        p.oneOrMoreTargets(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toThenSection(section phase4.Section) *ast.ThenSection {
	return &ast.ThenSection{
		Clauses:        p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

//////////////////////////////////////////// if ////////////////////////////////////////////////////

func (p *parser) toIfGroup(group phase4.Group) (ast.IfGroup, bool) {
	if !startsWithSections(group, ast.LowerIfName) {
		return ast.IfGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.IfSections...)
	if !ok {
		return ast.IfGroup{}, false
	}
	return ast.IfGroup{
		Label:          label,
		If:             *p.toIfSection(sections[ast.LowerIfName]),
		Then:           *p.toThenSection(sections[ast.LowerThenName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toIfSection(section phase4.Section) *ast.IfSection {
	return &ast.IfSection{
		Clauses:        p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toElseIfSection(section phase4.Section) *ast.ElseIfSection {
	return &ast.ElseIfSection{
		Clauses:        p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

/////////////////////////////////////////// iff ////////////////////////////////////////////////////

func (p *parser) toIffGroup(group phase4.Group) (ast.IffGroup, bool) {
	if !startsWithSections(group, ast.LowerIffName) {
		return ast.IffGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.IffSections...)
	if !ok {
		return ast.IffGroup{}, false
	}
	return ast.IffGroup{
		Label:          label,
		Iff:            *p.toIffSection(sections[ast.LowerIffName]),
		Then:           *p.toThenSection(sections[ast.LowerThenName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toIffSection(section phase4.Section) *ast.IffSection {
	return &ast.IffSection{
		Clauses:        p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////// when ////////////////////////////////////////////////////

func (p *parser) toWhenGroup(group phase4.Group) (ast.WhenGroup, bool) {
	if !startsWithSections(group, ast.LowerWhenName) {
		return ast.WhenGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.WhenSections...)
	if !ok {
		return ast.WhenGroup{}, false
	}
	return ast.WhenGroup{
		Label:          label,
		When:           *p.toWhenSection(sections[ast.LowerWhenName]),
		Then:           *p.toThenSection(sections[ast.LowerThenName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toWhenSection(section phase4.Section) *ast.WhenSection {
	return &ast.WhenSection{
		When:           p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////// piecewise ///////////////////////////////////////////////////

func (p *parser) toPiecewiseGroup(group phase4.Group) (ast.PiecewiseGroup, bool) {
	if !startsWithSections(group, ast.LowerPiecewiseName) {
		return ast.PiecewiseGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections := group.Sections

	if len(sections) == 0 || sections[0].Name != ast.LowerPiecewiseName {
		return ast.PiecewiseGroup{}, false
	}
	piecewise := *p.toPiecewiseSection(sections[0])

	if len(sections) < 3 ||
		sections[1].Name != ast.LowerIfName ||
		sections[2].Name != ast.LowerThenName {
		return ast.PiecewiseGroup{}, false
	}

	ifThen := ast.IfThen{
		If:   *p.toIfSection(sections[1]),
		Then: *p.toThenSection(sections[2]),
	}

	elseIfThens := make([]ast.ElseIfThen, 0)
	i := 3 // the first three sections are piecewise, if, and then
	for i < len(sections) {
		section1 := sections[i]
		if section1.Name == ast.LowerElseName {
			break
		}
		i++
		if section1.Name != ast.LowerElseIfName {
			p.tracker.Append(p.newError(fmt.Sprintf("Expected section '%s' but found '%s'",
				ast.LowerElseIfName, section1.Name), section1.MetaData.Start))
			return ast.PiecewiseGroup{}, false
		}
		if i >= len(sections) {
			p.tracker.Append(p.newError(fmt.Sprintf("Expected section '%s' to follow an '%s' section",
				ast.LowerThenName, ast.LowerElseIfName), section1.MetaData.Start))
			return ast.PiecewiseGroup{}, false
		}
		section2 := sections[i]
		i++
		if section2.Name != ast.LowerThenName {
			p.tracker.Append(p.newError(fmt.Sprintf("Expected section '%s' but found '%s'",
				ast.LowerThenName, section2.Name), section2.MetaData.Start))
			return ast.PiecewiseGroup{}, false
		}
		elseIfThens = append(elseIfThens, ast.ElseIfThen{
			ElseIf: *p.toElseIfSection(section1),
			Then:   *p.toThenSection(section2),
		})
	}
	var elseSec *ast.ElseSection
	if i < len(sections) && sections[i].Name == ast.LowerElseName {
		elseSec = p.toElseSection(sections[i])
		i++
	}
	invalid := i < len(sections)
	for i < len(sections) {
		sec := sections[i]
		i++
		p.tracker.Append(p.newError(
			fmt.Sprintf("Unexpected section '%s'", sec.Name), sec.MetaData.Start))
	}
	if invalid {
		return ast.PiecewiseGroup{}, false
	}
	return ast.PiecewiseGroup{
		Label:          label,
		Piecewise:      piecewise,
		IfThen:         ifThen,
		ElseIfThen:     elseIfThens,
		Else:           elseSec,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toPiecewiseSection(section phase4.Section) *ast.PiecewiseSection {
	p.verifyNoArgs(section)
	return &ast.PiecewiseSection{CommonMetaData: toCommonMetaData(section.MetaData)}
}

func (p *parser) toElseSection(section phase4.Section) *ast.ElseSection {
	return &ast.ElseSection{
		Items:          p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

//////////////////////////////////////// provides //////////////////////////////////////////////////

func (p *parser) toSymbolWrittenGroup(group phase4.Group) (ast.SymbolWrittenGroup, bool) {
	if !startsWithSections(group, ast.LowerSymbolName) {
		return ast.SymbolWrittenGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.SymbolWrittenSections...)
	if !ok {
		return ast.SymbolWrittenGroup{}, false
	}
	symbol := *p.toSymbolSection(sections[ast.LowerSymbolName])
	var written *ast.WrittenSection
	if sect, ok := sections[ast.LowerWrittenName]; ok {
		written = p.toWrittenSection(sect)
	}
	return ast.SymbolWrittenGroup{
		Label:          label,
		Symbol:         symbol,
		Written:        written,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toSymbolSection(section phase4.Section) *ast.SymbolSection {
	return &ast.SymbolSection{
		Symbol:         p.exactlyOneAlias(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toWrittenSection(section phase4.Section) *ast.WrittenSection {
	return &ast.WrittenSection{
		Written:        p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toEncodingGroup(group phase4.Group) (ast.EncodingGroup, bool) {
	if !startsWithSections(group, ast.LowerEncodingName) {
		return ast.EncodingGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.EncodingSections...)
	if !ok {
		return ast.EncodingGroup{}, false
	}

	encoding := *p.toEncodingSection(sections[ast.LowerEncodingName])
	as := *p.toAsSection(sections[ast.LowerAsName])
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sect, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sect)
	}
	var through *ast.ThroughSection
	if sect, ok := sections[ast.LowerThroughName]; ok {
		through = p.toThroughSection(sect)
	}
	return ast.EncodingGroup{
		Label:          label,
		Encoding:       encoding,
		As:             as,
		Using:          using,
		Where:          where,
		Through:        through,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toEncodingSection(section phase4.Section) *ast.EncodingSection {
	p.verifyNoArgs(section)
	return &ast.EncodingSection{
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toViewGroup(group phase4.Group) (ast.ViewGroup, bool) {
	if !startsWithSections(group, ast.LowerViewName) {
		return ast.ViewGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ViewSections...)
	if !ok {
		return ast.ViewGroup{}, false
	}

	view := *p.toViewSection(sections[ast.LowerViewName])
	as := *p.toAsSection(sections[ast.LowerAsName])
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sect, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sect)
	}
	var through *ast.ThroughSection
	if sect, ok := sections[ast.LowerThroughName]; ok {
		through = p.toThroughSection(sect)
	}
	var signifies *ast.SignifiesSection
	if sect, ok := sections[ast.LowerSignifiesName]; ok {
		signifies = p.toSignifiesSection(sect)
	}
	return ast.ViewGroup{
		Label:          label,
		View:           view,
		As:             as,
		Using:          using,
		Where:          where,
		Through:        through,
		Signfies:       signifies,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toViewSection(section phase4.Section) *ast.ViewSection {
	p.verifyNoArgs(section)
	return &ast.ViewSection{
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toAsSection(section phase4.Section) *ast.AsSection {
	return &ast.AsSection{
		As:             p.exactlyOneTarget(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toSignifiesSection(section phase4.Section) *ast.SignifiesSection {
	return &ast.SignifiesSection{
		Signifies:      p.oneOrMoreSpecs(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toThroughSection(section phase4.Section) *ast.ThroughSection {
	return &ast.ThroughSection{
		Through:        p.oneOrMoreFormulation(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

/////////////////////////////// codified documented items //////////////////////////////////////////

func (p *parser) toWrittenGroup(group phase4.Group) (ast.WrittenGroup, bool) {
	if !startsWithSections(group, ast.LowerWrittenName) {
		return ast.WrittenGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.WrittenSections...)
	if !ok {
		return ast.WrittenGroup{}, false
	}
	return ast.WrittenGroup{
		Label:          label,
		Written:        *p.toWrittenSection(sections[ast.LowerWrittenName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toCalledGroup(group phase4.Group) (ast.CalledGroup, bool) {
	if !startsWithSections(group, ast.LowerCalledName) {
		return ast.CalledGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.CalledSections...)
	if !ok {
		return ast.CalledGroup{}, false
	}
	return ast.CalledGroup{
		Label:          label,
		Called:         *p.toCalledSection(sections[ast.LowerCalledName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toCalledSection(section phase4.Section) *ast.CalledSection {
	return &ast.CalledSection{
		Called:         p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toWritingGroup(group phase4.Group) (ast.WritingGroup, bool) {
	if !startsWithSections(group, ast.LowerWritingName) {
		return ast.WritingGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.WritingSections...)
	if !ok {
		return ast.WritingGroup{}, false
	}
	return ast.WritingGroup{
		Label:          label,
		Writing:        *p.toWritingSection(sections[ast.LowerWritingName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toWritingSection(section phase4.Section) *ast.WritingSection {
	return &ast.WritingSection{
		Writing:        p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////// documented ///////////////////////////////////////////////

func (p *parser) toDocumentedSection(section phase4.Section) *ast.DocumentedSection {
	return &ast.DocumentedSection{
		Documented:     p.oneOrMoreDocumentedKinds(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toDocumentedKind(arg phase4.Argument) (ast.DocumentedKind, bool) {
	switch group := arg.Arg.(type) {
	case *phase4.Group:
		if grp, ok := p.toOverviewGroup(*group); ok {
			return &grp, true
		} else if grp, ok := p.toRelatedGroup(*group); ok {
			return &grp, true
		} else if grp, ok := p.toWrittenGroup(*group); ok {
			return &grp, true
		} else if grp, ok := p.toWritingGroup(*group); ok {
			return &grp, true
		} else if grp, ok := p.toCalledGroup(*group); ok {
			return &grp, true
		}
	}

	return nil, false
}

func (p *parser) toOverviewGroup(group phase4.Group) (ast.OverviewGroup, bool) {
	if !startsWithSections(group, ast.LowerOverviewName) {
		return ast.OverviewGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.OverviewSections...)
	if !ok {
		return ast.OverviewGroup{}, false
	}
	return ast.OverviewGroup{
		Label:          label,
		Overview:       *p.toOverviewSection(sections[ast.LowerOverviewName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toOverviewSection(section phase4.Section) *ast.OverviewSection {
	return &ast.OverviewSection{
		Overview:       p.exactlyOneTextItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toRelatedGroup(group phase4.Group) (ast.RelatedGroup, bool) {
	if !startsWithSections(group, ast.LowerRelatedName) {
		return ast.RelatedGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.RelatedSections...)
	if !ok {
		return ast.RelatedGroup{}, false
	}
	return ast.RelatedGroup{
		Label:          label,
		Related:        *p.toRelatedSection(sections[ast.LowerRelatedName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toRelatedSection(section phase4.Section) *ast.RelatedSection {
	return &ast.RelatedSection{
		Related:        p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////// provides ////////////////////////////////////////////////////

func (p *parser) toProvidesSection(section phase4.Section) *ast.ProvidesSection {
	return &ast.ProvidesSection{
		Provides:       p.oneOrMoreProvidesKind(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) oneOrMoreProvidesKind(section phase4.Section) []ast.ProvidesKind {
	return oneOrMore(p, p.toProvidesKinds(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) toProvidesKinds(args []phase4.Argument) []ast.ProvidesKind {
	result := make([]ast.ProvidesKind, 0)
	for _, arg := range args {
		if providesType, ok := p.toProvidesKindFromArg(arg); ok {
			result = append(result, providesType)
		}
	}
	return result
}

func (p *parser) toProvidesKindFromArg(arg phase4.Argument) (ast.ProvidesKind, bool) {
	if _, ok := arg.Arg.(*phase4.FormulationArgumentData); ok {
		alias := p.toAlias(arg)
		return &alias, true
	}

	group, ok := arg.Arg.(*phase4.Group)
	if !ok {
		return nil, false
	}
	return p.toProvidesKindFromGroup(*group)
}

func (p *parser) toProvidesKindFromGroup(group phase4.Group) (ast.ProvidesKind, bool) {
	if grp, ok := p.toSymbolWrittenGroup(group); ok {
		return &grp, true
	} else if grp, ok := p.toViewGroup(group); ok {
		return &grp, ok
	} else if grp, ok := p.toEncodingGroup(group); ok {
		return &grp, ok
	} else {
		p.tracker.Append(p.newError(fmt.Sprintf("Unrecognized argument for %s:\n"+
			"Expected one of:\n\n%s:\n\n%s:\n",
			ast.UpperProvidesName,
			ast.LowerOperationsName,
			ast.LowerMembersName), group.Start()))
		return nil, false
	}
}

///////////////////////////////////////// aliases //////////////////////////////////////////////////

func (p *parser) toAliasesSection(section phase4.Section) *ast.AliasesSection {
	return &ast.AliasesSection{
		Aliases:        p.oneOrMoreAliases(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

/////////////////////////////////////////// justified //////////////////////////////////////////////

func (p *parser) toJustifiedSection(section phase4.Section) *ast.JustifiedSection {
	return &ast.JustifiedSection{
		Justified:      p.oneOrMoreJustifiedKind(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) oneOrMoreJustifiedKind(section phase4.Section) []ast.JustifiedKind {
	return oneOrMore(p, p.toJustifiedKinds(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) toJustifiedKinds(args []phase4.Argument) []ast.JustifiedKind {
	result := make([]ast.JustifiedKind, 0)
	for _, arg := range args {
		if justifiedType, ok := p.toJustifiedKindFromArg(arg); ok {
			result = append(result, justifiedType)
		}
	}
	return result
}

func (p *parser) toJustifiedKindFromArg(arg phase4.Argument) (ast.JustifiedKind, bool) {
	group, ok := arg.Arg.(*phase4.Group)
	if !ok {
		return nil, false
	}
	return p.toJustifiedKindFromGroup(*group)
}

func (p *parser) toJustifiedKindFromGroup(group phase4.Group) (ast.JustifiedKind, bool) {
	if grp, ok := p.toLabelGroup(group); ok {
		return &grp, true
	} else if grp, ok := p.toByGroup(group); ok {
		return &grp, true
	} else {
		return nil, false
	}
}

func (p *parser) toLabelSection(section phase4.Section) *ast.LabelSection {
	return &ast.LabelSection{
		Label:          p.exactlyOneTextItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toLabelGroup(group phase4.Group) (ast.LabelGroup, bool) {
	if !startsWithSections(group, ast.LowerLabelName) {
		return ast.LabelGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker,
		ast.LowerLabelName,
		ast.LowerByName)
	if !ok {
		return ast.LabelGroup{}, false
	}
	return ast.LabelGroup{
		Label:          *p.toLabelSection(sections[ast.LowerLabelName]),
		By:             *p.toBySection(sections[ast.LowerByName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toByGroup(group phase4.Group) (ast.ByGroup, bool) {
	if !startsWithSections(group, ast.LowerByName) {
		return ast.ByGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.LowerByName)
	if !ok {
		return ast.ByGroup{}, false
	}
	return ast.ByGroup{
		By:             *p.toBySection(sections[ast.LowerByName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toBySection(section phase4.Section) *ast.BySection {
	return &ast.BySection{
		By:             p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////// references //////////////////////////////////////////////////

func (p *parser) toReferencesSection(section phase4.Section) *ast.ReferencesSection {
	return &ast.ReferencesSection{
		References:     p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////// meta id //////////////////////////////////////////////////

func (p *parser) toMetaIdSection(section phase4.Section) *ast.MetaIdSection {
	return &ast.MetaIdSection{
		Id:             p.exactlyOneTextItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////// describes ///////////////////////////////////////////////////

func (p *parser) toDescribesGroup(group phase4.Group) (ast.DescribesGroup, bool) {
	if !startsWithSections(group, ast.UpperDescribesName) {
		return ast.DescribesGroup{}, false
	}

	id := p.getId(group, true)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.DescribesSections...)
	if !ok || id == nil {
		return ast.DescribesGroup{}, false
	}
	describes := *p.toDescribesSection(sections[ast.UpperDescribesName])
	var using *ast.UsingSection
	if sec, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sec)
	}
	var when *ast.WhenSection
	if sec, ok := sections[ast.LowerWhenName]; ok {
		when = p.toWhenSection(sec)
	}
	var suchThat *ast.SuchThatSection
	if sec, ok := sections[ast.LowerSuchThatName]; ok {
		suchThat = p.toSuchThatSection(sec)
	}
	var extends *ast.ExtendsSection
	if sec, ok := sections[ast.LowerExtendsName]; ok {
		extends = p.toExtendsSection(sec)
	}
	var satisfies *ast.SatisfiesSection
	if sec, ok := sections[ast.LowerSatisfiesName]; ok {
		satisfies = p.toSatisfiesSection(sec)
	}
	var provides *ast.ProvidesSection
	if sec, ok := sections[ast.UpperProvidesName]; ok {
		provides = p.toProvidesSection(sec)
	}
	var justified *ast.JustifiedSection
	if sec, ok := sections[ast.UpperJustifiedName]; ok {
		justified = p.toJustifiedSection(sec)
	}
	var documented *ast.DocumentedSection
	if sec, ok := sections[ast.UpperDocumentedName]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections[ast.UpperReferencesName]; ok {
		references = p.toReferencesSection(sec)
	}
	var aliases *ast.AliasesSection
	if sec, ok := sections[ast.UpperAliasesName]; ok {
		aliases = p.toAliasesSection(sec)
	}
	var metaId *ast.MetaIdSection
	if sec, ok := sections[ast.UpperIdName]; ok {
		metaId = p.toMetaIdSection(sec)
	}
	return ast.DescribesGroup{
		Id:             *id,
		Describes:      describes,
		Using:          using,
		When:           when,
		SuchThat:       suchThat,
		Extends:        extends,
		Satisfies:      satisfies,
		Provides:       provides,
		Justified:      justified,
		Documented:     documented,
		References:     references,
		Aliases:        aliases,
		MetaId:         metaId,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toDescribesSection(section phase4.Section) *ast.DescribesSection {
	return &ast.DescribesSection{
		Describes:      p.exactlyOneTarget(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toExtendsSection(section phase4.Section) *ast.ExtendsSection {
	return &ast.ExtendsSection{
		Extends:        p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toSatisfiesSection(section phase4.Section) *ast.SatisfiesSection {
	return &ast.SatisfiesSection{
		Satisfies:      p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

/////////////////////////////////////////// lower defines //////////////////////////////////////////

func (p *parser) toLowerDefineGroup(group phase4.Group) (ast.LowerDefineGroup, bool) {
	if !startsWithSections(group, ast.LowerDefineName) {
		return ast.LowerDefineGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.LowerDefineSections...)
	if !ok {
		return ast.LowerDefineGroup{}, false
	}
	define := *p.toLowerDefineSection(sections[ast.LowerDefineName])
	var using *ast.UsingSection
	if sec, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sec)
	}
	var when *ast.WhenSection
	if sec, ok := sections[ast.LowerWhenName]; ok {
		when = p.toWhenSection(sec)
	}
	var suchThat *ast.SuchThatSection
	if sec, ok := sections[ast.LowerSuchThatName]; ok {
		suchThat = p.toSuchThatSection(sec)
	}
	var means *ast.MeansSection
	if sec, ok := sections[ast.LowerMeansName]; ok {
		means = p.toMeansSection(sec)
	}
	as := *p.toDefineAsSection(sections[ast.LowerAsName])
	return ast.LowerDefineGroup{
		Define:         define,
		Using:          using,
		When:           when,
		SuchThat:       suchThat,
		Means:          means,
		As:             as,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toLowerDefineSection(section phase4.Section) *ast.LowerDefineSection {
	return &ast.LowerDefineSection{
		Define:         p.exactlyOneTarget(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toDefineAsSection(section phase4.Section) *ast.DefineAsSection {
	return &ast.DefineAsSection{
		As:             p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////////// defines //////////////////////////////////////////////

func (p *parser) toDefinesGroup(group phase4.Group) (ast.DefinesGroup, bool) {
	if !startsWithSections(group, ast.UpperDefinesName) {
		return ast.DefinesGroup{}, false
	}

	id := p.getId(group, true)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.DefinesSections...)
	if !ok || id == nil {
		return ast.DefinesGroup{}, false
	}
	defines := *p.toDefinesSection(sections[ast.UpperDefinesName])
	var using *ast.UsingSection
	if sec, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sec)
	}
	var when *ast.WhenSection
	if sec, ok := sections[ast.LowerWhenName]; ok {
		when = p.toWhenSection(sec)
	}
	var suchThat *ast.SuchThatSection
	if sec, ok := sections[ast.LowerSuchThatName]; ok {
		suchThat = p.toSuchThatSection(sec)
	}
	var means *ast.MeansSection
	if sec, ok := sections[ast.LowerMeansName]; ok {
		means = p.toMeansSection(sec)
	}
	var specifies *ast.SpecifiesSection
	if sec, ok := sections[ast.LowerSpecifiesName]; ok {
		specifies = p.toSpecifiesSection(sec)
	}
	var provides *ast.ProvidesSection
	if sec, ok := sections[ast.UpperProvidesName]; ok {
		provides = p.toProvidesSection(sec)
	}
	var justified *ast.JustifiedSection
	if sec, ok := sections[ast.UpperJustifiedName]; ok {
		justified = p.toJustifiedSection(sec)
	}
	var documented *ast.DocumentedSection
	if sec, ok := sections[ast.UpperDocumentedName]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections[ast.UpperReferencesName]; ok {
		references = p.toReferencesSection(sec)
	}
	var aliases *ast.AliasesSection
	if sec, ok := sections[ast.UpperAliasesName]; ok {
		aliases = p.toAliasesSection(sec)
	}
	var metaId *ast.MetaIdSection
	if sec, ok := sections[ast.UpperIdName]; ok {
		metaId = p.toMetaIdSection(sec)
	}
	return ast.DefinesGroup{
		Id:             *id,
		Defines:        defines,
		Using:          using,
		When:           when,
		SuchThat:       suchThat,
		Means:          means,
		Specifies:      specifies,
		Provides:       provides,
		Justified:      justified,
		Documented:     documented,
		References:     references,
		Aliases:        aliases,
		MetaId:         metaId,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toDefinesSection(section phase4.Section) *ast.DefinesSection {
	return &ast.DefinesSection{
		Defines:        p.exactlyOneTarget(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toSingleMeansSection(section phase4.Section) *ast.SingleMeansSection {
	return &ast.SingleMeansSection{
		Means:          p.exactlyOneClause(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toMeansSection(section phase4.Section) *ast.MeansSection {
	return &ast.MeansSection{
		Means:          p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toSpecifiesSection(section phase4.Section) *ast.SpecifiesSection {
	return &ast.SpecifiesSection{
		Specifies:      p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////// captures ////////////////////////////////////////////////

func (p *parser) toCapturesGroup(group phase4.Group) (ast.CapturesGroup, bool) {
	if !startsWithSections(group, ast.UpperCapturesName) {
		return ast.CapturesGroup{}, false
	}

	id := p.getId(group, true)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.CapturesSections...)
	if !ok || id == nil {
		return ast.CapturesGroup{}, false
	}
	captures := *p.toCapturesSection(sections[ast.UpperCapturesName])
	var justified *ast.JustifiedSection
	if sec, ok := sections[ast.UpperJustifiedName]; ok {
		justified = p.toJustifiedSection(sec)
	}
	var documented *ast.DocumentedSection
	if sec, ok := sections[ast.UpperDocumentedName]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections[ast.UpperReferencesName]; ok {
		references = p.toReferencesSection(sec)
	}
	var metaId *ast.MetaIdSection
	if sec, ok := sections[ast.UpperIdName]; ok {
		metaId = p.toMetaIdSection(sec)
	}
	return ast.CapturesGroup{
		Id:             *id,
		Captures:       captures,
		Justified:      justified,
		Documented:     documented,
		References:     references,
		MetaId:         metaId,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toCapturesSection(section phase4.Section) *ast.CapturesSection {
	return &ast.CapturesSection{
		Captures:       p.oneOrMoreFormulation(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

/////////////////////////////////////////// states /////////////////////////////////////////////////

func (p *parser) toStatesGroup(group phase4.Group) (ast.StatesGroup, bool) {
	if !startsWithSections(group, ast.UpperStatesName) {
		return ast.StatesGroup{}, false
	}

	id := p.getId(group, true)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.StatesSections...)
	if !ok || id == nil {
		return ast.StatesGroup{}, false
	}
	states := *p.toStatesSection(sections[ast.UpperStatesName])
	var using *ast.UsingSection
	if sec, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sec)
	}
	var when *ast.WhenSection
	if sec, ok := sections[ast.LowerWhenName]; ok {
		when = p.toWhenSection(sec)
	}
	var suchThat *ast.SuchThatSection
	if sec, ok := sections[ast.LowerSuchThatName]; ok {
		suchThat = p.toSuchThatSection(sec)
	}
	that := *p.toThatSection(sections[ast.LowerThatName])
	var justified *ast.JustifiedSection
	if sec, ok := sections[ast.UpperJustifiedName]; ok {
		justified = p.toJustifiedSection(sec)
	}
	var documented *ast.DocumentedSection
	if sec, ok := sections[ast.UpperDocumentedName]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections[ast.UpperReferencesName]; ok {
		references = p.toReferencesSection(sec)
	}
	var aliases *ast.AliasesSection
	if sec, ok := sections[ast.UpperAliasesName]; ok {
		aliases = p.toAliasesSection(sec)
	}
	var metaId *ast.MetaIdSection
	if sec, ok := sections[ast.UpperIdName]; ok {
		metaId = p.toMetaIdSection(sec)
	}
	return ast.StatesGroup{
		Id:             *id,
		States:         states,
		Using:          using,
		When:           when,
		SuchThat:       suchThat,
		That:           that,
		Justified:      justified,
		Documented:     documented,
		References:     references,
		Aliases:        aliases,
		MetaId:         metaId,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toStatesSection(section phase4.Section) *ast.StatesSection {
	p.verifyNoArgs(section)
	return &ast.StatesSection{
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toThatSection(section phase4.Section) *ast.ThatSection {
	return &ast.ThatSection{
		That:           p.oneOrMoreClauses(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////// axiom ////////////////////////////////////////////////////

func (p *parser) toAxiomGroup(group phase4.Group) (ast.AxiomGroup, bool) {
	if !startsWithSections(group, ast.UpperAxiomName) {
		return ast.AxiomGroup{}, false
	}

	id := p.getId(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.AxiomSections...)
	if !ok {
		return ast.AxiomGroup{}, false
	}
	axiom := *p.toAxiomSection(sections[ast.UpperAxiomName])
	var given *ast.GivenSection
	if sec, ok := sections[ast.LowerGivenName]; ok {
		given = p.toGivenSection(sec)
	}
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sec, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sec)
	}
	var ifSec *ast.IfSection
	if sec, ok := sections[ast.LowerIfName]; ok {
		ifSec = p.toIfSection(sec)
	}
	var iff *ast.IffSection
	if sec, ok := sections[ast.LowerIffName]; ok {
		iff = p.toIffSection(sec)
	}
	then := *p.toThenSection(sections[ast.LowerThenName])
	var documented *ast.DocumentedSection
	if sec, ok := sections[ast.UpperDocumentedName]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections[ast.UpperReferencesName]; ok {
		references = p.toReferencesSection(sec)
	}
	var aliases *ast.AliasesSection
	if sec, ok := sections[ast.UpperAliasesName]; ok {
		aliases = p.toAliasesSection(sec)
	}
	var metaId *ast.MetaIdSection
	if sec, ok := sections[ast.UpperIdName]; ok {
		metaId = p.toMetaIdSection(sec)
	}
	return ast.AxiomGroup{
		Id:             id,
		Axiom:          axiom,
		Given:          given,
		Using:          using,
		Where:          where,
		If:             ifSec,
		Iff:            iff,
		Then:           then,
		Documented:     documented,
		References:     references,
		Aliases:        aliases,
		MetaId:         metaId,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toAxiomSection(section phase4.Section) *ast.AxiomSection {
	p.verifyNoArgs(section)
	return &ast.AxiomSection{
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toGivenSection(section phase4.Section) *ast.GivenSection {
	return &ast.GivenSection{
		Given:          p.oneOrMoreTargets(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toUsingSection(section phase4.Section) *ast.UsingSection {
	return &ast.UsingSection{
		Using:          p.oneOrMoreTargets(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////// conjecture ///////////////////////////////////////////////

func (p *parser) toConjectureGroup(group phase4.Group) (ast.ConjectureGroup, bool) {
	if !startsWithSections(group, ast.UpperConjectureName) {
		return ast.ConjectureGroup{}, false
	}

	id := p.getId(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ConjectureSections...)
	if !ok {
		return ast.ConjectureGroup{}, false
	}
	conjecture := *p.toConjectureSection(sections[ast.UpperConjectureName])
	var given *ast.GivenSection
	if sec, ok := sections[ast.LowerGivenName]; ok {
		given = p.toGivenSection(sec)
	}
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sec, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sec)
	}
	var ifSec *ast.IfSection
	if sec, ok := sections[ast.LowerIfName]; ok {
		ifSec = p.toIfSection(sec)
	}
	var iff *ast.IffSection
	if sec, ok := sections[ast.LowerIffName]; ok {
		iff = p.toIffSection(sec)
	}
	then := *p.toThenSection(sections[ast.LowerThenName])
	var documented *ast.DocumentedSection
	if sec, ok := sections[ast.UpperDocumentedName]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections[ast.UpperReferencesName]; ok {
		references = p.toReferencesSection(sec)
	}
	var aliases *ast.AliasesSection
	if sec, ok := sections[ast.UpperAliasesName]; ok {
		aliases = p.toAliasesSection(sec)
	}
	var metaId *ast.MetaIdSection
	if sec, ok := sections[ast.UpperIdName]; ok {
		metaId = p.toMetaIdSection(sec)
	}
	return ast.ConjectureGroup{
		Id:             id,
		Conjecture:     conjecture,
		Given:          given,
		Using:          using,
		Where:          where,
		If:             ifSec,
		Iff:            iff,
		Then:           then,
		Documented:     documented,
		References:     references,
		Aliases:        aliases,
		MetaId:         metaId,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toConjectureSection(section phase4.Section) *ast.ConjectureSection {
	p.verifyNoArgs(section)
	return &ast.ConjectureSection{
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////// theorem /////////////////////////////////////////////////

func (p *parser) toTheoremGroup(group phase4.Group) (ast.TheoremGroup, bool) {
	if !startsWithSections(group, ast.UpperTheoremName) {
		return ast.TheoremGroup{}, false
	}

	id := p.getId(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.TheoremSections...)
	if !ok {
		return ast.TheoremGroup{}, false
	}
	theorem := *p.toTheoremSection(sections[ast.UpperTheoremName])
	var given *ast.GivenSection
	if sec, ok := sections[ast.LowerGivenName]; ok {
		given = p.toGivenSection(sec)
	}
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sec, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sec)
	}
	var ifSec *ast.IfSection
	if sec, ok := sections[ast.LowerIfName]; ok {
		ifSec = p.toIfSection(sec)
	}
	var iff *ast.IffSection
	if sec, ok := sections[ast.LowerIffName]; ok {
		iff = p.toIffSection(sec)
	}
	then := *p.toThenSection(sections[ast.LowerThenName])
	var proof *ast.ProofSection
	if sec, ok := sections[ast.UpperProofName]; ok {
		proof = p.toProofSection(sec)
	}
	var documented *ast.DocumentedSection
	if sec, ok := sections[ast.UpperDocumentedName]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections[ast.UpperReferencesName]; ok {
		references = p.toReferencesSection(sec)
	}
	var aliases *ast.AliasesSection
	if sec, ok := sections[ast.UpperAliasesName]; ok {
		aliases = p.toAliasesSection(sec)
	}
	var metaId *ast.MetaIdSection
	if sec, ok := sections[ast.UpperIdName]; ok {
		metaId = p.toMetaIdSection(sec)
	}
	return ast.TheoremGroup{
		Id:             id,
		Theorem:        theorem,
		Given:          given,
		Using:          using,
		Where:          where,
		If:             ifSec,
		Iff:            iff,
		Then:           then,
		Proof:          proof,
		Documented:     documented,
		References:     references,
		Aliases:        aliases,
		MetaId:         metaId,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toTheoremSection(section phase4.Section) *ast.TheoremSection {
	p.verifyNoArgs(section)
	return &ast.TheoremSection{
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toProofSection(section phase4.Section) *ast.ProofSection {
	return &ast.ProofSection{
		Proof:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////// corollary ///////////////////////////////////////////////

func (p *parser) toCorollaryGroup(group phase4.Group) (ast.CorollaryGroup, bool) {
	if !startsWithSections(group, ast.UpperCorollaryName) {
		return ast.CorollaryGroup{}, false
	}

	id := p.getId(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.CorollarySections...)
	if !ok {
		return ast.CorollaryGroup{}, false
	}
	corollary := *p.toCorollarySection(sections[ast.UpperCorollaryName])
	to := *p.toCorollaryToSection(sections[ast.LowerToName])
	var given *ast.GivenSection
	if sec, ok := sections[ast.LowerGivenName]; ok {
		given = p.toGivenSection(sec)
	}
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sec, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sec)
	}
	var ifSec *ast.IfSection
	if sec, ok := sections[ast.LowerIfName]; ok {
		ifSec = p.toIfSection(sec)
	}
	var iff *ast.IffSection
	if sec, ok := sections[ast.LowerIffName]; ok {
		iff = p.toIffSection(sec)
	}
	then := *p.toThenSection(sections[ast.LowerThenName])
	var documented *ast.DocumentedSection
	if sec, ok := sections[ast.UpperDocumentedName]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections[ast.UpperReferencesName]; ok {
		references = p.toReferencesSection(sec)
	}
	var aliases *ast.AliasesSection
	if sec, ok := sections[ast.UpperAliasesName]; ok {
		aliases = p.toAliasesSection(sec)
	}
	var metaId *ast.MetaIdSection
	if sec, ok := sections[ast.UpperIdName]; ok {
		metaId = p.toMetaIdSection(sec)
	}
	return ast.CorollaryGroup{
		Id:             id,
		Corollary:      corollary,
		To:             to,
		Given:          given,
		Using:          using,
		Where:          where,
		If:             ifSec,
		Iff:            iff,
		Then:           then,
		Documented:     documented,
		References:     references,
		Aliases:        aliases,
		MetaId:         metaId,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toCorollarySection(section phase4.Section) *ast.CorollarySection {
	p.verifyNoArgs(section)
	return &ast.CorollarySection{
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toCorollaryToSection(section phase4.Section) *ast.CorollaryToSection {
	return &ast.CorollaryToSection{
		To:             p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////// lemma ///////////////////////////////////////////////

func (p *parser) toLemmaGroup(group phase4.Group) (ast.LemmaGroup, bool) {
	if !startsWithSections(group, ast.UpperLemmaName) {
		return ast.LemmaGroup{}, false
	}

	id := p.getId(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.LemmaSections...)
	if !ok {
		return ast.LemmaGroup{}, false
	}
	lemma := *p.toLemmaSection(sections[ast.UpperLemmaName])
	forSection := *p.toLemmaForSection(sections[ast.LowerForName])
	var given *ast.GivenSection
	if sec, ok := sections[ast.LowerGivenName]; ok {
		given = p.toGivenSection(sec)
	}
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sec, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sec)
	}
	var ifSec *ast.IfSection
	if sec, ok := sections[ast.LowerIfName]; ok {
		ifSec = p.toIfSection(sec)
	}
	var iff *ast.IffSection
	if sec, ok := sections[ast.LowerIffName]; ok {
		iff = p.toIffSection(sec)
	}
	then := *p.toThenSection(sections[ast.LowerThenName])
	var documented *ast.DocumentedSection
	if sec, ok := sections[ast.UpperDocumentedName]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections[ast.UpperReferencesName]; ok {
		references = p.toReferencesSection(sec)
	}
	var aliases *ast.AliasesSection
	if sec, ok := sections[ast.UpperAliasesName]; ok {
		aliases = p.toAliasesSection(sec)
	}
	var metaId *ast.MetaIdSection
	if sec, ok := sections[ast.UpperIdName]; ok {
		metaId = p.toMetaIdSection(sec)
	}
	return ast.LemmaGroup{
		Id:             id,
		Lemma:          lemma,
		For:            forSection,
		Given:          given,
		Using:          using,
		Where:          where,
		If:             ifSec,
		Iff:            iff,
		Then:           then,
		Documented:     documented,
		References:     references,
		Aliases:        aliases,
		MetaId:         metaId,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toLemmaSection(section phase4.Section) *ast.LemmaSection {
	p.verifyNoArgs(section)
	return &ast.LemmaSection{
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toLemmaForSection(section phase4.Section) *ast.LemmaForSection {
	return &ast.LemmaForSection{
		For:            p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////// text blocks /////////////////////////////////////////////

func (p *parser) toTextBlockItem(block phase4.TextBlock) *ast.TextBlockItem {
	return &ast.TextBlockItem{
		Text:           block.Text,
		CommonMetaData: toCommonMetaData(block.MetaData),
	}
}

////////////////////////////////////////// specify /////////////////////////////////////////////////

func (p *parser) toSpecifyGroup(group phase4.Group) (ast.SpecifyGroup, bool) {
	if !startsWithSections(group, ast.UpperSpecifyName) {
		return ast.SpecifyGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.SpecifySections...)
	if !ok {
		return ast.SpecifyGroup{}, false
	}

	var metaId *ast.MetaIdSection
	if sec, ok := sections[ast.UpperIdName]; ok {
		metaId = p.toMetaIdSection(sec)
	}

	return ast.SpecifyGroup{
		Specify:        *p.toTopLevelSpecifySection(sections[ast.UpperSpecifyName]),
		MetaId:         metaId,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toTopLevelSpecifySection(section phase4.Section) *ast.TopLevelSpecifySection {
	return &ast.TopLevelSpecifySection{
		Specify:        p.oneOrMoreSpecifyKinds(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toSpecifyKind(arg phase4.Argument) (ast.SpecifyKind, bool) {
	switch group := arg.Arg.(type) {
	case *phase4.Group:
		if grp, ok := p.toZeroGroup(*group); ok {
			return &grp, true
		} else if grp, ok := p.toPositiveIntGroup(*group); ok {
			return &grp, true
		} else if grp, ok := p.toNegativeIntGroup(*group); ok {
			return &grp, true
		} else if grp, ok := p.toPositiveFloatGroup(*group); ok {
			return &grp, true
		} else if grp, ok := p.toNegativeFloatGroup(*group); ok {
			return &grp, true
		}
	}
	return nil, false
}

func (p *parser) toZeroGroup(group phase4.Group) (ast.ZeroGroup, bool) {
	if !startsWithSections(group, ast.LowerZeroName) {
		return ast.ZeroGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ZeroSections...)
	if !ok {
		return ast.ZeroGroup{}, false
	}
	return ast.ZeroGroup{
		Zero:           *p.toZeroSection(sections[ast.LowerZeroName]),
		SingleMeans:    *p.toSingleMeansSection(sections[ast.LowerMeansName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toZeroSection(section phase4.Section) *ast.ZeroSection {
	p.verifyNoArgs(section)
	return &ast.ZeroSection{
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toPositiveIntGroup(group phase4.Group) (ast.PositiveIntGroup, bool) {
	if !startsWithSections(group, ast.LowerPositiveIntName) {
		return ast.PositiveIntGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.PositiveIntSections...)
	if !ok {
		return ast.PositiveIntGroup{}, false
	}
	return ast.PositiveIntGroup{
		PositiveInt:    *p.toPositiveIntSection(sections[ast.LowerPositiveIntName]),
		SingleMeans:    *p.toSingleMeansSection(sections[ast.LowerMeansName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toPositiveIntSection(section phase4.Section) *ast.PositiveIntSection {
	return &ast.PositiveIntSection{
		PositiveInt:    p.exactlyOneTarget(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toNegativeIntGroup(group phase4.Group) (ast.NegativeIntGroup, bool) {
	if !startsWithSections(group, ast.LowerNegativeIntName) {
		return ast.NegativeIntGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.NegativeIntSections...)
	if !ok {
		return ast.NegativeIntGroup{}, false
	}
	return ast.NegativeIntGroup{
		NegativeInt:    *p.toNegativeIntSection(sections[ast.LowerNegativeIntName]),
		SingleMeans:    *p.toSingleMeansSection(sections[ast.LowerMeansName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toNegativeIntSection(section phase4.Section) *ast.NegativeIntSection {
	return &ast.NegativeIntSection{
		NegativeInt:    p.exactlyOneTarget(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toPositiveFloatGroup(group phase4.Group) (ast.PositiveFloatGroup, bool) {
	if !startsWithSections(group, ast.LowerPositiveFloatName) {
		return ast.PositiveFloatGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.PositiveFloatSections...)
	if !ok {
		return ast.PositiveFloatGroup{}, false
	}
	return ast.PositiveFloatGroup{
		PositiveFloat:  *p.toPositiveFloatSection(sections[ast.LowerPositiveFloatName]),
		SingleMeans:    *p.toSingleMeansSection(sections[ast.LowerMeansName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toPositiveFloatSection(section phase4.Section) *ast.PositiveFloatSection {
	return &ast.PositiveFloatSection{
		PositiveFloat:  p.exactlyOneTarget(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toNegativeFloatGroup(group phase4.Group) (ast.NegativeFloatGroup, bool) {
	if !startsWithSections(group, ast.LowerNegativeFloatName) {
		return ast.NegativeFloatGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.NegativeFloatSections...)
	if !ok {
		return ast.NegativeFloatGroup{}, false
	}
	return ast.NegativeFloatGroup{
		NegativeFloat:  *p.toNegativeFloatSection(sections[ast.LowerNegativeFloatName]),
		SingleMeans:    *p.toSingleMeansSection(sections[ast.LowerMeansName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toNegativeFloatSection(section phase4.Section) *ast.NegativeFloatSection {
	return &ast.NegativeFloatSection{
		NegativeFloat:  p.exactlyOneTarget(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toResourceGroup(group phase4.Group) (ast.ResourceGroup, bool) {
	if !startsWithSections(group, ast.UpperResourceName) {
		return ast.ResourceGroup{}, false
	}

	id := p.getStringId(group, true)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ResourceSections...)
	if !ok || id == nil {
		return ast.ResourceGroup{}, false
	}

	var metaId *ast.MetaIdSection
	if sec, ok := sections[ast.UpperIdName]; ok {
		metaId = p.toMetaIdSection(sec)
	}

	return ast.ResourceGroup{
		Id:             *id,
		Resource:       *p.toResourceSection(sections[ast.UpperResourceName]),
		MetaId:         metaId,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toResourceSection(section phase4.Section) *ast.ResourceSection {
	return &ast.ResourceSection{
		Items:          p.oneOrMoreResourceKinds(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toResourceKind(arg phase4.Argument) (ast.ResourceKind, bool) {
	switch group := arg.Arg.(type) {
	case *phase4.Group:
		if grp, ok := p.toTitleGroup(*group); ok {
			return &grp, ok
		} else if grp, ok := p.toAuthorGroup(*group); ok {
			return &grp, ok
		} else if grp, ok := p.toOffsetGroup(*group); ok {
			return &grp, ok
		} else if grp, ok := p.toUrlGroup(*group); ok {
			return &grp, ok
		} else if grp, ok := p.toHomepageGroup(*group); ok {
			return &grp, ok
		} else if grp, ok := p.toTypeGroup(*group); ok {
			return &grp, ok
		} else if grp, ok := p.toEditionGroup(*group); ok {
			return &grp, ok
		} else if grp, ok := p.toEditorGroup(*group); ok {
			return &grp, ok
		} else if grp, ok := p.toInstitutionGroup(*group); ok {
			return &grp, ok
		} else if grp, ok := p.toJournalGroup(*group); ok {
			return &grp, ok
		} else if grp, ok := p.toPublisherGroup(*group); ok {
			return &grp, ok
		} else if grp, ok := p.toVolumeGroup(*group); ok {
			return &grp, ok
		} else if grp, ok := p.toMonthGroup(*group); ok {
			return &grp, ok
		} else if grp, ok := p.toYearGroup(*group); ok {
			return &grp, ok
		} else if grp, ok := p.toDescriptionGroup(*group); ok {
			return &grp, ok
		}
	}
	return nil, false
}

func (p *parser) toPersonGroup(group phase4.Group) (ast.PersonGroup, bool) {
	if !startsWithSections(group, ast.UpperPersonName) {
		return ast.PersonGroup{}, false
	}

	id := p.getStringId(group, true)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.PersonSections...)
	if !ok || id == nil {
		return ast.PersonGroup{}, false
	}

	var metaId *ast.MetaIdSection
	if sec, ok := sections[ast.UpperIdName]; ok {
		metaId = p.toMetaIdSection(sec)
	}

	return ast.PersonGroup{
		Id:             *id,
		Person:         *p.toPersonSection(sections[ast.UpperPersonName]),
		MetaId:         metaId,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toPersonSection(section phase4.Section) *ast.PersonSection {
	return &ast.PersonSection{
		Items:          p.oneOrMorePersonKinds(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toTitleGroup(group phase4.Group) (ast.TitleGroup, bool) {
	if !startsWithSections(group, ast.LowerTitleName) {
		return ast.TitleGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.TitleSections...)
	if !ok {
		return ast.TitleGroup{}, false
	}
	return ast.TitleGroup{
		Title:          *p.toTitleSection(sections[ast.LowerTitleName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toTitleSection(section phase4.Section) *ast.TitleSection {
	return &ast.TitleSection{
		Title:          p.exactlyOneTextItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toAuthorGroup(group phase4.Group) (ast.AuthorGroup, bool) {
	if !startsWithSections(group, ast.LowerAuthorName) {
		return ast.AuthorGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.AuthorSections...)
	if !ok {
		return ast.AuthorGroup{}, false
	}
	return ast.AuthorGroup{
		Author:         *p.toAuthorSection(sections[ast.LowerAuthorName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toAuthorSection(section phase4.Section) *ast.AuthorSection {
	return &ast.AuthorSection{
		Author:         p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toOffsetGroup(group phase4.Group) (ast.OffsetGroup, bool) {
	if !startsWithSections(group, ast.LowerOffsetName) {
		return ast.OffsetGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.OffsetSections...)
	if !ok {
		return ast.OffsetGroup{}, false
	}
	return ast.OffsetGroup{
		Offset:         *p.toOffsetSection(sections[ast.LowerOffsetName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toOffsetSection(section phase4.Section) *ast.OffsetSection {
	return &ast.OffsetSection{
		Offset:         p.exactlyOneTextItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toUrlGroup(group phase4.Group) (ast.UrlGroup, bool) {
	if !startsWithSections(group, ast.LowerUrlName) {
		return ast.UrlGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.UrlSections...)
	if !ok {
		return ast.UrlGroup{}, false
	}
	return ast.UrlGroup{
		Url:            *p.toUlrSection(sections[ast.LowerUrlName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toUlrSection(section phase4.Section) *ast.UrlSection {
	return &ast.UrlSection{
		Url:            p.exactlyOneTextItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toHomepageGroup(group phase4.Group) (ast.HomepageGroup, bool) {
	if !startsWithSections(group, ast.LowerHomepageName) {
		return ast.HomepageGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.HomepageSections...)
	if !ok {
		return ast.HomepageGroup{}, false
	}
	return ast.HomepageGroup{
		Homepage:       *p.toHomepageSection(sections[ast.LowerHomepageName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toHomepageSection(section phase4.Section) *ast.HomepageSection {
	return &ast.HomepageSection{
		Homepage:       p.exactlyOneTextItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toTypeGroup(group phase4.Group) (ast.TypeGroup, bool) {
	if !startsWithSections(group, ast.LowerTypeName) {
		return ast.TypeGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.TypeSections...)
	if !ok {
		return ast.TypeGroup{}, false
	}
	return ast.TypeGroup{
		Type:           *p.toTypeSection(sections[ast.LowerTypeName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toTypeSection(section phase4.Section) *ast.TypeSection {
	return &ast.TypeSection{
		Type:           p.exactlyOneTextItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toEditionGroup(group phase4.Group) (ast.EditionGroup, bool) {
	if !startsWithSections(group, ast.LowerEditionName) {
		return ast.EditionGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.EditionSections...)
	if !ok {
		return ast.EditionGroup{}, false
	}
	return ast.EditionGroup{
		Edition:        *p.toEditionSection(sections[ast.LowerEditionName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toEditionSection(section phase4.Section) *ast.EditionSection {
	return &ast.EditionSection{
		Edition:        p.exactlyOneTextItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toEditorGroup(group phase4.Group) (ast.EditorGroup, bool) {
	if !startsWithSections(group, ast.LowerEditorName) {
		return ast.EditorGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.EditorSections...)
	if !ok {
		return ast.EditorGroup{}, false
	}
	return ast.EditorGroup{
		Editor:         *p.toEditorSection(sections[ast.LowerEditorName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toEditorSection(section phase4.Section) *ast.EditorSection {
	return &ast.EditorSection{
		Editor:         p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toInstitutionGroup(group phase4.Group) (ast.InstitutionGroup, bool) {
	if !startsWithSections(group, ast.LowerInstitutionName) {
		return ast.InstitutionGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.InstitutionSections...)
	if !ok {
		return ast.InstitutionGroup{}, false
	}
	return ast.InstitutionGroup{
		Institution:    *p.toInstitutionSection(sections[ast.LowerInstitutionName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toInstitutionSection(section phase4.Section) *ast.InstitutionSection {
	return &ast.InstitutionSection{
		Institution:    p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toJournalGroup(group phase4.Group) (ast.JournalGroup, bool) {
	if !startsWithSections(group, ast.LowerJournalName) {
		return ast.JournalGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.JournalSections...)
	if !ok {
		return ast.JournalGroup{}, false
	}
	return ast.JournalGroup{
		Journal:        *p.toJournalSection(sections[ast.LowerJournalName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toJournalSection(section phase4.Section) *ast.JournalSection {
	return &ast.JournalSection{
		Journal:        p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toPublisherGroup(group phase4.Group) (ast.PublisherGroup, bool) {
	if !startsWithSections(group, ast.LowerPublisherName) {
		return ast.PublisherGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.PublisherSections...)
	if !ok {
		return ast.PublisherGroup{}, false
	}
	return ast.PublisherGroup{
		Publisher:      *p.toPublisherSection(sections[ast.LowerPublisherName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toPublisherSection(section phase4.Section) *ast.PublisherSection {
	return &ast.PublisherSection{
		Publisher:      p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toVolumeGroup(group phase4.Group) (ast.VolumeGroup, bool) {
	if !startsWithSections(group, ast.LowerVolumeName) {
		return ast.VolumeGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.VolumeSections...)
	if !ok {
		return ast.VolumeGroup{}, false
	}
	return ast.VolumeGroup{
		Volume:         *p.toVolumeSection(sections[ast.LowerVolumeName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toVolumeSection(section phase4.Section) *ast.VolumeSection {
	return &ast.VolumeSection{
		Volume:         p.exactlyOneTextItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toMonthGroup(group phase4.Group) (ast.MonthGroup, bool) {
	if !startsWithSections(group, ast.LowerMonthName) {
		return ast.MonthGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.MonthSections...)
	if !ok {
		return ast.MonthGroup{}, false
	}
	return ast.MonthGroup{
		Month:          *p.toMonthSection(sections[ast.LowerMonthName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toMonthSection(section phase4.Section) *ast.MonthSection {
	return &ast.MonthSection{
		Month:          p.exactlyOneTextItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toYearGroup(group phase4.Group) (ast.YearGroup, bool) {
	if !startsWithSections(group, ast.LowerYearName) {
		return ast.YearGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.YearSections...)
	if !ok {
		return ast.YearGroup{}, false
	}
	return ast.YearGroup{
		Year:           *p.toYearSection(sections[ast.LowerYearName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toYearSection(section phase4.Section) *ast.YearSection {
	return &ast.YearSection{
		Year:           p.exactlyOneTextItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toDescriptionGroup(group phase4.Group) (ast.DescriptionGroup, bool) {
	if !startsWithSections(group, ast.LowerDescriptionName) {
		return ast.DescriptionGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.DescriptionSections...)
	if !ok {
		return ast.DescriptionGroup{}, false
	}
	return ast.DescriptionGroup{
		Description:    *p.toDescriptionSection(sections[ast.LowerDescriptionName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toDescriptionSection(section phase4.Section) *ast.DescriptionSection {
	return &ast.DescriptionSection{
		Description:    p.exactlyOneTextItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////// top level items //////////////////////////////////////////

func (p *parser) toTopLevelItemKind(item phase4.TopLevelNodeKind) (ast.TopLevelItemKind, bool) {
	switch item := item.(type) {
	case *phase4.TextBlock:
		return p.toTextBlockItem(*item), true
	case *phase4.Group:
		if grp, ok := p.toDefinesGroup(*item); ok {
			return &grp, true
		} else if grp, ok := p.toDescribesGroup(*item); ok {
			return &grp, true
		} else if grp, ok := p.toCapturesGroup(*item); ok {
			return &grp, true
		} else if grp, ok := p.toStatesGroup(*item); ok {
			return &grp, true
		} else if grp, ok := p.toAxiomGroup(*item); ok {
			return &grp, true
		} else if grp, ok := p.toConjectureGroup(*item); ok {
			return &grp, true
		} else if grp, ok := p.toTheoremGroup(*item); ok {
			return &grp, true
		} else if grp, ok := p.toCorollaryGroup(*item); ok {
			return &grp, true
		} else if grp, ok := p.toLemmaGroup(*item); ok {
			return &grp, true
		} else if grp, ok := p.toSpecifyGroup(*item); ok {
			return &grp, true
		} else if grp, ok := p.toPersonGroup(*item); ok {
			return &grp, ok
		} else if grp, ok := p.toResourceGroup(*item); ok {
			return &grp, ok
		}
	}
	p.tracker.Append(p.newError("Invalid top level item", item.Start()))
	return nil, false
}

/////////////////////////////////////// document ///////////////////////////////////////////////////

func (p *parser) toDocument(root phase4.Document) (ast.Document, bool) {
	countBefore := p.tracker.Length()
	items := make([]ast.TopLevelItemKind, 0)
	for _, node := range root.Nodes {
		if item, ok := p.toTopLevelItemKind(node); ok {
			items = append(items, item)
		}
	}
	start := ast.Position{
		Row:    0,
		Column: 0,
		Offset: 0,
	}
	if len(items) > 0 {
		start = items[0].GetCommonMetaData().Start
	}
	return ast.Document{
		Items: items,
		CommonMetaData: ast.CommonMetaData{
			Start: start,
			Key:   p.keyGen.Next(),
		},
	}, p.tracker.Length() == countBefore
}

///////////////////////////////////////////// id ///////////////////////////////////////////////////

func (p *parser) toIdItem(text string, position ast.Position) *ast.IdItem {
	if node, ok := formulation.ParseId(p.path, text, position, p.tracker, p.keyGen); ok {
		return &ast.IdItem{
			RawText: text,
			Root:    node,
		}
	} else {
		return nil
	}
}

func (p *parser) getId(group phase4.Group, required bool) *ast.IdItem {
	if required && group.Id == nil {
		p.tracker.Append(p.newError("Expected a [...] item", group.MetaData.Start))
		return nil
	}
	if group.Id == nil {
		return nil
	}
	return p.toIdItem(*group.Id, group.MetaData.Start)
}

func (p *parser) getGroupLabel(group phase4.Group, required bool) *ast.GroupLabel {
	if required && group.Id == nil {
		p.tracker.Append(p.newError("Expected a [...] item", group.MetaData.Start))
		return nil
	}
	if group.Id == nil {
		return nil
	}
	return &ast.GroupLabel{
		Label: *group.Id,
		Start: group.MetaData.Start,
	}
}

func (p *parser) getStringId(group phase4.Group, required bool) *string {
	if required && group.Id == nil {
		p.tracker.Append(p.newError("Expected a [...] item", group.MetaData.Start))
		return nil
	} else if group.Id == nil {
		return nil
	}
	return group.Id
}

//////////////////////////////////////// arguments /////////////////////////////////////////////////

func (p *parser) maybeToFormulation(
	arg phase4.Argument,
) (ast.Formulation[ast.FormulationNodeKind], bool) {
	switch data := arg.Arg.(type) {
	case *phase4.FormulationArgumentData:
		if node, ok := formulation.ParseExpression(
			p.path, data.Text, arg.MetaData.Start, p.tracker, p.keyGen); ok {
			return ast.Formulation[ast.FormulationNodeKind]{
				RawText:        data.Text,
				Root:           node,
				Label:          data.Label,
				CommonMetaData: toCommonMetaData(data.MetaData),
			}, true
		}
	}
	return ast.Formulation[ast.FormulationNodeKind]{}, false
}

func (p *parser) toFormulation(arg phase4.Argument) ast.Formulation[ast.FormulationNodeKind] {
	if formulation, ok := p.maybeToFormulation(arg); ok {
		return formulation
	}
	p.tracker.Append(p.newError("Expected a formulation", arg.MetaData.Start))
	return ast.Formulation[ast.FormulationNodeKind]{}
}

func (p *parser) toClause(arg phase4.Argument) ast.ClauseKind {
	switch data := arg.Arg.(type) {
	case *phase4.TextArgumentData:
		return &ast.TextItem{
			RawText:        data.Text,
			CommonMetaData: toCommonMetaData(arg.MetaData),
		}
	case *phase4.FormulationArgumentData:
		if node, ok := formulation.ParseExpression(
			p.path, data.Text, arg.MetaData.Start, p.tracker, p.keyGen); ok {
			return &ast.Formulation[ast.FormulationNodeKind]{
				RawText:        data.Text,
				Root:           node,
				Label:          data.Label,
				CommonMetaData: toCommonMetaData(data.MetaData),
			}
		} else {
			return &ast.Formulation[ast.FormulationNodeKind]{}
		}
	case *phase4.Group:
		if grp, ok := p.toAllOfGroup(*data); ok {
			return &grp
		} else if grp, ok := p.toEquivalentlyGroup(*data); ok {
			return &grp
		} else if grp, ok := p.toNotGroup(*data); ok {
			return &grp
		} else if grp, ok := p.toAnyOfGroup(*data); ok {
			return &grp
		} else if grp, ok := p.toOneOfGroup(*data); ok {
			return &grp
		} else if grp, ok := p.toForAllGroup(*data); ok {
			return &grp
		} else if grp, ok := p.toExistsGroup(*data); ok {
			return &grp
		} else if grp, ok := p.toExistsUniqueGroup(*data); ok {
			return &grp
		} else if grp, ok := p.toIfGroup(*data); ok {
			return &grp
		} else if grp, ok := p.toIffGroup(*data); ok {
			return &grp
		} else if grp, ok := p.toWhenGroup(*data); ok {
			return &grp
		} else if grp, ok := p.toPiecewiseGroup(*data); ok {
			return &grp
		} else if grp, ok := p.toLetGroup(*data); ok {
			return &grp
		} else if grp, ok := p.toLowerDefineGroup(*data); ok {
			return &grp
		}
	}

	p.tracker.Append(p.newError(
		fmt.Sprintf("Expected a '...', `...`, %s:, %s:, %s:, %s:, or %s: item",
			ast.LowerExistsName, ast.LowerExistsUniqueName, ast.LowerForAllName, ast.LowerIfName,
			ast.LowerIffName), arg.MetaData.Start))
	return &ast.Formulation[ast.FormulationNodeKind]{}
}

func (p *parser) toSpec(arg phase4.Argument) ast.Spec {
	switch data := arg.Arg.(type) {
	case *phase4.FormulationArgumentData:
		if node, ok := formulation.ParseExpression(
			p.path, data.Text, arg.MetaData.Start, p.tracker, p.keyGen); ok {
			return ast.Spec{
				RawText:        data.Text,
				Root:           node,
				Label:          data.Label,
				CommonMetaData: toCommonMetaData(data.MetaData),
			}
		} else {
			return ast.Spec{}
		}
	default:
		p.tracker.Append(p.newError(
			"Expected a '... is ...' or a '... <op> ...' item", arg.MetaData.Start))
		return ast.Spec{}
	}
}

func (p *parser) toAlias(arg phase4.Argument) ast.Alias {
	switch data := arg.Arg.(type) {
	case *phase4.FormulationArgumentData:
		if node, ok := formulation.ParseExpression(
			p.path, data.Text, arg.MetaData.Start, p.tracker, p.keyGen); ok {
			return ast.Alias{
				RawText:        data.Text,
				Root:           node,
				Label:          data.Label,
				CommonMetaData: toCommonMetaData(data.MetaData),
			}
		}
	}
	p.tracker.Append(p.newError("Expected a '... :=> ...' or '... :-> ...' item", arg.MetaData.Start))
	return ast.Alias{}
}

func (p *parser) toTarget(arg phase4.Argument) ast.Target {
	switch data := arg.Arg.(type) {
	case *phase4.ArgumentTextArgumentData:
		if node, ok := formulation.ParseForm(p.path, data.Text, arg.MetaData.Start,
			p.tracker, p.keyGen); ok {
			return ast.Target{
				RawText:        data.Text,
				Root:           node,
				CommonMetaData: toCommonMetaData(data.MetaData),
			}
		} else {
			return ast.Target{}
		}
	default:
		p.tracker.Append(p.newError(
			"Expected a name, function, set, tuple, or ':=' declaration", arg.MetaData.Start))
		return ast.Target{}
	}
}

func (p *parser) maybeToTextItem(arg phase4.Argument) (ast.TextItem, bool) {
	switch data := arg.Arg.(type) {
	case *phase4.TextArgumentData:
		return ast.TextItem{
			RawText:        data.Text,
			CommonMetaData: toCommonMetaData(data.MetaData),
		}, true
	default:
		return ast.TextItem{}, false
	}
}

func (p *parser) toTextItem(arg phase4.Argument) ast.TextItem {
	item, ok := p.maybeToTextItem(arg)
	if !ok {
		p.tracker.Append(p.newError("Expected a \"...\" item", arg.MetaData.Start))
	}
	return item
}

func (p *parser) toPersonKind(arg phase4.Argument) (ast.PersonKind, bool) {
	switch group := arg.Arg.(type) {
	case *phase4.Group:
		if grp, ok := p.toNameGroup(*group); ok {
			return &grp, true
		} else if grp, ok := p.toBiographyGroup(*group); ok {
			return &grp, true
		}
	}
	return nil, false
}

func (p *parser) toNameGroup(group phase4.Group) (ast.NameGroup, bool) {
	if !startsWithSections(group, ast.LowerNameName) {
		return ast.NameGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.NameSections...)
	if !ok {
		return ast.NameGroup{}, false
	}

	return ast.NameGroup{
		Name: *p.toNameSection(sections[ast.LowerNameName]),
	}, true
}

func (p *parser) toNameSection(section phase4.Section) *ast.NameSection {
	return &ast.NameSection{
		Name:           p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toBiographyGroup(group phase4.Group) (ast.BiographyGroup, bool) {
	if !startsWithSections(group, ast.LowerBiographyName) {
		return ast.BiographyGroup{}, false
	}

	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.BiographySections...)
	if !ok {
		return ast.BiographyGroup{}, false
	}

	return ast.BiographyGroup{
		Biography: *p.toBiographySection(sections[ast.LowerBiographyName]),
	}, true
}

func (p *parser) toBiographySection(section phase4.Section) *ast.BiographySection {
	return &ast.BiographySection{
		Biography:      p.exactlyOneTextItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toSignatureItem(arg phase4.Argument) ast.Formulation[*ast.Signature] {
	switch data := arg.Arg.(type) {
	case *phase4.FormulationArgumentData:
		if node, ok := formulation.ParseSignature(
			p.path, data.Text, arg.MetaData.Start, p.tracker, p.keyGen); ok {
			return ast.Formulation[*ast.Signature]{
				RawText:        data.Text,
				Root:           &node,
				Label:          data.Label,
				CommonMetaData: toCommonMetaData(data.MetaData),
			}
		} else {
			return ast.Formulation[*ast.Signature]{}
		}
	}

	p.tracker.Append(p.newError("Expected a signature", arg.MetaData.Start))
	return ast.Formulation[*ast.Signature]{}
}

///////////////////////////////////// argument lists ///////////////////////////////////////////////

func (p *parser) toFormulations(args []phase4.Argument) []ast.Formulation[ast.FormulationNodeKind] {
	result := make([]ast.Formulation[ast.FormulationNodeKind], 0)
	for _, arg := range args {
		result = append(result, p.toFormulation(arg))
	}
	return result
}

func (p *parser) toClauses(args []phase4.Argument) []ast.ClauseKind {
	result := make([]ast.ClauseKind, 0)
	for _, arg := range args {
		result = append(result, p.toClause(arg))
	}
	return result
}

func (p *parser) toProofItems(args []phase4.Argument) []ast.ProofItemKind {
	result := make([]ast.ProofItemKind, 0)
	for _, arg := range args {
		result = append(result, p.toProofItem(arg))
	}
	return result
}

func (p *parser) toSpecs(args []phase4.Argument) []ast.Spec {
	result := make([]ast.Spec, 0)
	for _, arg := range args {
		result = append(result, p.toSpec(arg))
	}
	return result
}

func (p *parser) toAliases(args []phase4.Argument) []ast.Alias {
	result := make([]ast.Alias, 0)
	for _, arg := range args {
		result = append(result, p.toAlias(arg))
	}
	return result
}

func (p *parser) toTargets(args []phase4.Argument) []ast.Target {
	result := make([]ast.Target, 0)
	for _, arg := range args {
		result = append(result, p.toTarget(arg))
	}
	return result
}

func (p *parser) toTextItems(args []phase4.Argument) []ast.TextItem {
	result := make([]ast.TextItem, 0)
	for _, arg := range args {
		result = append(result, p.toTextItem(arg))
	}
	return result
}

func (p *parser) toPersonKinds(args []phase4.Argument) []ast.PersonKind {
	result := make([]ast.PersonKind, 0)
	for _, arg := range args {
		if note, ok := p.toPersonKind(arg); ok {
			result = append(result, note)
		}
	}
	return result
}

func (p *parser) toResourceKinds(args []phase4.Argument) []ast.ResourceKind {
	result := make([]ast.ResourceKind, 0)
	for _, arg := range args {
		if note, ok := p.toResourceKind(arg); ok {
			result = append(result, note)
		}
	}
	return result
}

func (p *parser) toSignaturesItems(args []phase4.Argument) []ast.Formulation[*ast.Signature] {
	result := make([]ast.Formulation[*ast.Signature], 0)
	for _, arg := range args {
		result = append(result, p.toSignatureItem(arg))
	}
	return result
}

func (p *parser) toDocumentedKinds(args []phase4.Argument) []ast.DocumentedKind {
	result := make([]ast.DocumentedKind, 0)
	for _, arg := range args {
		if doc, ok := p.toDocumentedKind(arg); ok {
			result = append(result, doc)
		} else {
			p.tracker.Append(p.newError(fmt.Sprintf(
				"Expected a %s:, %s:, %s:, %s:, or %s: item",
				ast.LowerOverviewName, ast.LowerRelatedName, ast.LowerWrittenName,
				ast.LowerWritingName, ast.LowerCalledName), arg.MetaData.Start))
		}
	}
	return result
}

func (p *parser) toSpecifyKinds(args []phase4.Argument) []ast.SpecifyKind {
	result := make([]ast.SpecifyKind, 0)
	for _, arg := range args {
		if spec, ok := p.toSpecifyKind(arg); ok {
			result = append(result, spec)
		} else {
			p.tracker.Append(p.newError(fmt.Sprintf(
				"Expected a %s:, %s:, %s:, %s:, or %s: item",
				ast.LowerZeroName,
				ast.LowerPositiveIntName,
				ast.LowerNegativeIntName,
				ast.LowerPositiveFloatName,
				ast.LowerNegativeFloatName), arg.MetaData.Start))
		}
	}
	return result
}

func (p *parser) toProofItem(arg phase4.Argument) ast.ProofItemKind {
	if item, ok := p.maybeToTextItem(arg); ok {
		return &item
	} else if formulation, ok := p.maybeToFormulation(arg); ok {
		return &formulation
	}
	switch group := arg.Arg.(type) {
	case *phase4.Group:
		if grp, ok := p.toProofEquivalentlyGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofAllOfGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofNotGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofAnyOfGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofOneOfGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofExistsGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofExistsUniqueGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofForAllGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofLetGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toLowerDefineGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofIfGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofIffGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofNextGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofThenGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofThusGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofThereforeGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofHenceGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofNoticeGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofNextGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofByBecauseThenGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofBecauseThenGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofStepwiseGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofSupposeGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofBlockGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofWithoutLossOfGeneralityGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofForContrapositiveGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofForContradictionGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofForInductionGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofClaimGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofCasewiseGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofAbsurdGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofQedGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofDoneGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofContradictionGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofPartwiseGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofSufficesToShowGroup(*group); ok {
			return &grp
		} else if grp, ok := p.toProofToShowGroup(*group); ok {
			return &grp
		}
	}

	p.tracker.Append(p.newError("Expected a proof item", arg.MetaData.Start))
	return &ast.TextItem{}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofNoticeSection(section phase4.Section) *ast.ProofNoticeSection {
	return &ast.ProofNoticeSection{
		Notice:         p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toProofThusSection(section phase4.Section) *ast.ProofThusSection {
	return &ast.ProofThusSection{
		Thus:           p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toProofThereforeSection(section phase4.Section) *ast.ProofThereforeSection {
	return &ast.ProofThereforeSection{
		Therefore:      p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toProofHenceSection(section phase4.Section) *ast.ProofHenceSection {
	return &ast.ProofHenceSection{
		Hence:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toProofNextSection(section phase4.Section) *ast.ProofNextSection {
	return &ast.ProofNextSection{
		Next:           p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofThenGroup(group phase4.Group) (ast.ProofThenGroup, bool) {
	if !startsWithSections(group, ast.LowerThenName) {
		return ast.ProofThenGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ProofThenSections...)
	if !ok {
		return ast.ProofThenGroup{}, false
	}
	then := *p.toProofThenSection(sections[ast.LowerThenName])
	var by *ast.ProofBySection
	if sec, ok := sections[ast.LowerByName]; ok {
		by = p.toProofBySection(sec)
	}
	var because *ast.ProofBecauseSection
	if sec, ok := sections[ast.LowerBecauseName]; ok {
		because = p.toProofBecauseSection(sec)
	}
	return ast.ProofThenGroup{
		Label:          label,
		Then:           then,
		By:             by,
		Because:        because,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofThusGroup(group phase4.Group) (ast.ProofThusGroup, bool) {
	if !startsWithSections(group, ast.LowerThusName) {
		return ast.ProofThusGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ProofThusSections...)
	if !ok {
		return ast.ProofThusGroup{}, false
	}
	thus := *p.toProofThusSection(sections[ast.LowerThusName])
	var by *ast.ProofBySection
	if sec, ok := sections[ast.LowerByName]; ok {
		by = p.toProofBySection(sec)
	}
	var because *ast.ProofBecauseSection
	if sec, ok := sections[ast.LowerBecauseName]; ok {
		because = p.toProofBecauseSection(sec)
	}
	return ast.ProofThusGroup{
		Label:          label,
		Thus:           thus,
		By:             by,
		Because:        because,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofThereforeGroup(group phase4.Group) (ast.ProofThereforeGroup, bool) {
	if !startsWithSections(group, ast.LowerThereforeName) {
		return ast.ProofThereforeGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker,
		ast.ProofThereforeSections...)
	if !ok {
		return ast.ProofThereforeGroup{}, false
	}
	therefore := *p.toProofThereforeSection(sections[ast.LowerThereforeName])
	var by *ast.ProofBySection
	if sec, ok := sections[ast.LowerByName]; ok {
		by = p.toProofBySection(sec)
	}
	var because *ast.ProofBecauseSection
	if sec, ok := sections[ast.LowerBecauseName]; ok {
		because = p.toProofBecauseSection(sec)
	}
	return ast.ProofThereforeGroup{
		Label:          label,
		Therefore:      therefore,
		By:             by,
		Because:        because,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofHenceGroup(group phase4.Group) (ast.ProofHenceGroup, bool) {
	if !startsWithSections(group, ast.LowerHenceName) {
		return ast.ProofHenceGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ProofHenceSections...)
	if !ok {
		return ast.ProofHenceGroup{}, false
	}
	hence := *p.toProofHenceSection(sections[ast.LowerHenceName])
	var by *ast.ProofBySection
	if sec, ok := sections[ast.LowerByName]; ok {
		by = p.toProofBySection(sec)
	}
	var because *ast.ProofBecauseSection
	if sec, ok := sections[ast.LowerBecauseName]; ok {
		because = p.toProofBecauseSection(sec)
	}
	return ast.ProofHenceGroup{
		Label:          label,
		Hence:          hence,
		By:             by,
		Because:        because,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofNoticeGroup(group phase4.Group) (ast.ProofNoticeGroup, bool) {
	if !startsWithSections(group, ast.LowerNoticeName) {
		return ast.ProofNoticeGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ProofNoticeSections...)
	if !ok {
		return ast.ProofNoticeGroup{}, false
	}
	notice := *p.toProofNoticeSection(sections[ast.LowerNoticeName])
	var by *ast.ProofBySection
	if sec, ok := sections[ast.LowerByName]; ok {
		by = p.toProofBySection(sec)
	}
	var because *ast.ProofBecauseSection
	if sec, ok := sections[ast.LowerBecauseName]; ok {
		because = p.toProofBecauseSection(sec)
	}
	return ast.ProofNoticeGroup{
		Label:          label,
		Notice:         notice,
		By:             by,
		Because:        because,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofNextGroup(group phase4.Group) (ast.ProofNextGroup, bool) {
	if !startsWithSections(group, ast.LowerNextName) {
		return ast.ProofNextGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ProofNextSections...)
	if !ok {
		return ast.ProofNextGroup{}, false
	}
	next := *p.toProofNextSection(sections[ast.LowerNextName])
	var by *ast.ProofBySection
	if sec, ok := sections[ast.LowerByName]; ok {
		by = p.toProofBySection(sec)
	}
	var because *ast.ProofBecauseSection
	if sec, ok := sections[ast.LowerBecauseName]; ok {
		because = p.toProofBecauseSection(sec)
	}
	return ast.ProofNextGroup{
		Label:          label,
		Next:           next,
		By:             by,
		Because:        because,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofByBecauseThenGroup(group phase4.Group) (ast.ProofByBecauseThenGroup, bool) {
	if !startsWithSections(group, ast.LowerByName, ast.LowerThenName) {
		return ast.ProofByBecauseThenGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker,
		ast.ProofByBecauseThenSections...)
	if !ok {
		return ast.ProofByBecauseThenGroup{}, false
	}
	by := *p.toProofBySection(sections[ast.LowerByName])
	var because *ast.ProofBecauseSection
	if sec, ok := sections[ast.LowerBecauseName]; ok {
		because = p.toProofBecauseSection(sec)
	}
	then := *p.toProofThenSection(sections[ast.LowerThenName])
	return ast.ProofByBecauseThenGroup{
		Label:          label,
		By:             by,
		Because:        because,
		Then:           then,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofBecauseThenGroup(group phase4.Group) (ast.ProofBecauseThenGroup, bool) {
	if !startsWithSections(group, ast.LowerBecauseName, ast.LowerThenName) {
		return ast.ProofBecauseThenGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker,
		ast.ProofBecauseThenSections...)
	if !ok {
		return ast.ProofBecauseThenGroup{}, false
	}
	because := *p.toProofBecauseSection(sections[ast.LowerBecauseName])
	then := *p.toProofThenSection(sections[ast.LowerThenName])
	return ast.ProofBecauseThenGroup{
		Label:          label,
		Because:        because,
		Then:           then,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofBySection(section phase4.Section) *ast.ProofBySection {
	return &ast.ProofBySection{
		Items:          p.oneOrMoreTextItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toProofBecauseSection(section phase4.Section) *ast.ProofBecauseSection {
	return &ast.ProofBecauseSection{
		Because:        p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofForContrapositiveGroup(
	group phase4.Group,
) (ast.ProofForContrapositiveGroup, bool) {
	if !startsWithSections(group, ast.LowerForContrapositiveName) {
		return ast.ProofForContrapositiveGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(
		p.path, group.Sections, p.tracker, ast.ProofForContrapositiveSections...)
	if !ok {
		return ast.ProofForContrapositiveGroup{}, false
	}
	forContrapositives := *p.toProofForContrapositiveSection(
		sections[ast.LowerForContrapositiveName])
	return ast.ProofForContrapositiveGroup{
		Label:             label,
		ForContrapositive: forContrapositives,
		CommonMetaData:    toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofForContrapositiveSection(
	section phase4.Section,
) *ast.ProofForContrapositiveSection {
	return &ast.ProofForContrapositiveSection{
		Items:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofStepwiseGroup(group phase4.Group) (ast.ProofStepwiseGroup, bool) {
	if !startsWithSections(group, ast.LowerStepwiseName) {
		return ast.ProofStepwiseGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(
		p.path, group.Sections, p.tracker, ast.ProofStepwiseSections...)
	if !ok {
		return ast.ProofStepwiseGroup{}, false
	}
	stepwise := *p.toProofStepwiseSection(sections[ast.LowerStepwiseName])
	return ast.ProofStepwiseGroup{
		Label:          label,
		Stepwise:       stepwise,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofStepwiseSection(section phase4.Section) *ast.ProofStepwiseSection {
	return &ast.ProofStepwiseSection{
		Stepwise:       p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofBlockGroup(group phase4.Group) (ast.ProofBlockGroup, bool) {
	if !startsWithSections(group, ast.LowerBlockName) {
		return ast.ProofBlockGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ProofBlockSections...)
	if !ok {
		return ast.ProofBlockGroup{}, false
	}
	block := *p.toProofBlockSection(sections[ast.LowerBlockName])
	return ast.ProofBlockGroup{
		Label:          label,
		Block:          block,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofBlockSection(section phase4.Section) *ast.ProofBlockSection {
	return &ast.ProofBlockSection{
		Block:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofSupposeGroup(group phase4.Group) (ast.ProofSupposeGroup, bool) {
	if !startsWithSections(group, ast.LowerSupposeName) {
		return ast.ProofSupposeGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ProofSupposeSections...)
	if !ok {
		return ast.ProofSupposeGroup{}, false
	}
	suppose := *p.toProofSupposeSection(sections[ast.LowerSupposeName])
	then := *p.toProofSupposeThenSection(sections[ast.LowerThenName])
	return ast.ProofSupposeGroup{
		Label:          label,
		Suppose:        suppose,
		Then:           then,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofSupposeSection(section phase4.Section) *ast.ProofSupposeSection {
	return &ast.ProofSupposeSection{
		Suppose:        p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toProofSupposeThenSection(section phase4.Section) *ast.ProofThenSection {
	return &ast.ProofThenSection{
		Then:           p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofWithoutLossOfGeneralityGroup(
	group phase4.Group,
) (ast.ProofWithoutLossOfGeneralityGroup, bool) {
	if !startsWithSections(group, ast.LowerWithoutLossOfGeneralityName) {
		return ast.ProofWithoutLossOfGeneralityGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(
		p.path, group.Sections, p.tracker, ast.ProofWithoutLossOfGeneralitySections...)
	if !ok {
		return ast.ProofWithoutLossOfGeneralityGroup{}, false
	}
	withoutLossOfGenerality := *p.toProofWithoutLossOfGeneralitySection(
		sections[ast.LowerWithoutLossOfGeneralityName])
	return ast.ProofWithoutLossOfGeneralityGroup{
		Label:                   label,
		WithoutLossOfGenerality: withoutLossOfGenerality,
		CommonMetaData:          toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofWithoutLossOfGeneralitySection(
	section phase4.Section,
) *ast.ProofWithoutLossOfGeneralitySection {
	return &ast.ProofWithoutLossOfGeneralitySection{
		Items:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofSufficesToShowGroup(
	group phase4.Group,
) (ast.ProofSufficesToShowGroup, bool) {
	if !startsWithSections(group, ast.LowerSufficesToShowName) {
		return ast.ProofSufficesToShowGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(
		p.path, group.Sections, p.tracker, ast.ProofSufficesToShowSections...)
	if !ok {
		return ast.ProofSufficesToShowGroup{}, false
	}
	sufficesToShow := *p.toProofSufficesToShowSection(sections[ast.LowerSufficesToShowName])
	return ast.ProofSufficesToShowGroup{
		Label:          label,
		SufficesToShow: sufficesToShow,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofSufficesToShowSection(
	section phase4.Section,
) *ast.ProofSufficesToShowSection {
	return &ast.ProofSufficesToShowSection{
		Items:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofToShowGroup(
	group phase4.Group,
) (ast.ProofToShowGroup, bool) {
	if !startsWithSections(group, ast.LowerToShowName) {
		return ast.ProofToShowGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(
		p.path, group.Sections, p.tracker, ast.ProofToShowSections...)
	if !ok {
		return ast.ProofToShowGroup{}, false
	}
	toShow := *p.toProofToShowSection(sections[ast.LowerToShowName])
	observe := *p.toProofObserveSection(sections[ast.LowerObserveName])
	return ast.ProofToShowGroup{
		Label:          label,
		ToShow:         toShow,
		Observe:        observe,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofToShowSection(
	section phase4.Section,
) *ast.ProofToShowSection {
	return &ast.ProofToShowSection{
		Items:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toProofObserveSection(
	section phase4.Section,
) *ast.ProofObserveSection {
	return &ast.ProofObserveSection{
		Items:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofQedGroup(group phase4.Group) (ast.ProofQedGroup, bool) {
	if !startsWithSections(group, ast.LowerQedName) {
		return ast.ProofQedGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(
		p.path, group.Sections, p.tracker, ast.ProofQedSections...)
	if !ok {
		return ast.ProofQedGroup{}, false
	}
	qed := *p.toProofQedSection(sections[ast.LowerQedName])
	return ast.ProofQedGroup{
		Label:          label,
		Qed:            qed,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofQedSection(
	section phase4.Section,
) *ast.ProofQedSection {
	p.verifyNoArgs(section)
	return &ast.ProofQedSection{CommonMetaData: toCommonMetaData(section.MetaData)}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofAbsurdGroup(group phase4.Group) (ast.ProofAbsurdGroup, bool) {
	if !startsWithSections(group, ast.LowerAbsurdName) {
		return ast.ProofAbsurdGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(
		p.path, group.Sections, p.tracker, ast.ProofAbsurdSections...)
	if !ok {
		return ast.ProofAbsurdGroup{}, false
	}
	absurd := *p.toProofAbsurdSection(sections[ast.LowerAbsurdName])
	return ast.ProofAbsurdGroup{
		Label:          label,
		Absurd:         absurd,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofAbsurdSection(
	section phase4.Section,
) *ast.ProofAbsurdSection {
	p.verifyNoArgs(section)
	return &ast.ProofAbsurdSection{CommonMetaData: toCommonMetaData(section.MetaData)}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofDoneGroup(group phase4.Group) (ast.ProofDoneGroup, bool) {
	if !startsWithSections(group, ast.LowerDoneName) {
		return ast.ProofDoneGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(
		p.path, group.Sections, p.tracker, ast.ProofDoneSections...)
	if !ok {
		return ast.ProofDoneGroup{}, false
	}
	done := *p.toProofDoneSection(sections[ast.LowerDoneName])
	return ast.ProofDoneGroup{
		Label:          label,
		Done:           done,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofDoneSection(
	section phase4.Section,
) *ast.ProofDoneSection {
	p.verifyNoArgs(section)
	return &ast.ProofDoneSection{CommonMetaData: toCommonMetaData(section.MetaData)}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofContradictionGroup(group phase4.Group) (ast.ProofContradictionGroup, bool) {
	if !startsWithSections(group, ast.LowerContradictionName) {
		return ast.ProofContradictionGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(
		p.path, group.Sections, p.tracker, ast.ProofContradictionSections...)
	if !ok {
		return ast.ProofContradictionGroup{}, false
	}
	contradiction := *p.toProofContradictionSection(sections[ast.LowerContradictionName])
	return ast.ProofContradictionGroup{
		Label:          label,
		Contradiction:  contradiction,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofContradictionSection(
	section phase4.Section,
) *ast.ProofContradictionSection {
	p.verifyNoArgs(section)
	return &ast.ProofContradictionSection{CommonMetaData: toCommonMetaData(section.MetaData)}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofForContradictionGroup(
	group phase4.Group,
) (ast.ProofForContradictionGroup, bool) {
	if !startsWithSections(group, ast.LowerForContradictionName) {
		return ast.ProofForContradictionGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(
		p.path, group.Sections, p.tracker, ast.ProofForContradictionSections...)
	if !ok {
		return ast.ProofForContradictionGroup{}, false
	}
	forContradiction := *p.toProofForContradictionSection(sections[ast.LowerForContradictionName])
	return ast.ProofForContradictionGroup{
		Label:            label,
		ForContradiction: forContradiction,
		CommonMetaData:   toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofForContradictionSection(
	section phase4.Section,
) *ast.ProofForContradictionSection {
	return &ast.ProofForContradictionSection{
		Items:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofForInductionGroup(group phase4.Group) (ast.ProofForInductionGroup, bool) {
	if !startsWithSections(group, ast.LowerForInductionName) {
		return ast.ProofForInductionGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(
		p.path, group.Sections, p.tracker, ast.ProofForInductionSections...)
	if !ok {
		return ast.ProofForInductionGroup{}, false
	}
	forInduction := *p.toProofForInductionSection(sections[ast.LowerForInductionName])
	return ast.ProofForInductionGroup{
		Label:          label,
		ForInduction:   forInduction,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofForInductionSection(section phase4.Section) *ast.ProofForInductionSection {
	return &ast.ProofForInductionSection{
		Items:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofClaimGroup(group phase4.Group) (ast.ProofClaimGroup, bool) {
	if !startsWithSections(group, ast.LowerClaimName) {
		return ast.ProofClaimGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ProofClaimSections...)
	if !ok {
		return ast.ProofClaimGroup{}, false
	}
	claim := *p.toProofClaimSection(sections[ast.LowerClaimName])
	var given *ast.GivenSection
	if sec, ok := sections[ast.LowerGivenName]; ok {
		given = p.toGivenSection(sec)
	}
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sec, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sec)
	}
	var ifSec *ast.IfSection
	if sec, ok := sections[ast.LowerIfName]; ok {
		ifSec = p.toIfSection(sec)
	}
	var iff *ast.IffSection
	if sec, ok := sections[ast.LowerIffName]; ok {
		iff = p.toIffSection(sec)
	}
	then := *p.toThenSection(sections[ast.LowerThenName])
	var proof *ast.ProofSection
	if sec, ok := sections[ast.UpperProofName]; ok {
		proof = p.toProofSection(sec)
	}
	return ast.ProofClaimGroup{
		Label:          label,
		Claim:          claim,
		Given:          given,
		Using:          using,
		Where:          where,
		If:             ifSec,
		Iff:            iff,
		Then:           then,
		Proof:          proof,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofClaimSection(section phase4.Section) *ast.ProofClaimSection {
	p.verifyNoArgs(section)
	return &ast.ProofClaimSection{
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofCasewiseGroup(group phase4.Group) (ast.ProofCasewiseGroup, bool) {
	if !startsWithSections(group, ast.LowerCasewiseName) {
		return ast.ProofCasewiseGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections := group.Sections
	if len(sections) == 0 || sections[0].Name != ast.LowerCasewiseName {
		return ast.ProofCasewiseGroup{}, false
	}
	casewise := *p.toProofCasewiseSection(sections[0])
	cases := make([]ast.ProofCaseSection, 0)
	i := 1
	for i < len(sections) {
		section := sections[i]
		i++
		if section.Name == ast.LowerElseName {
			break
		}
		if section.Name != ast.LowerCaseName {
			p.tracker.Append(p.newError(fmt.Sprintf("Expected section '%s' but found '%s'",
				ast.LowerCaseName, section.Name), section.MetaData.Start))
			return ast.ProofCasewiseGroup{}, false
		}
		cases = append(cases, *p.toProofCaseSection(section))
	}
	var elseSection *ast.ProofElseSection
	if i < len(sections) && sections[i].Name == ast.LowerElseName {
		section := sections[i]
		i++
		elseSection = p.toProofElseSection(section)
	}
	for i < len(sections) {
		section := sections[i]
		i++
		p.tracker.Append(p.newError(fmt.Sprintf("Unexpected section '%s'", section.Name),
			section.MetaData.Start))
	}
	return ast.ProofCasewiseGroup{
		Label:          label,
		Casewise:       casewise,
		Cases:          cases,
		Else:           elseSection,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofCasewiseSection(section phase4.Section) *ast.ProofCasewiseSection {
	p.verifyNoArgs(section)
	return &ast.ProofCasewiseSection{CommonMetaData: toCommonMetaData(section.MetaData)}
}

func (p *parser) toProofCaseSection(section phase4.Section) *ast.ProofCaseSection {
	return &ast.ProofCaseSection{
		Case:           p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

func (p *parser) toProofElseSection(section phase4.Section) *ast.ProofElseSection {
	return &ast.ProofElseSection{
		Else:           p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) toProofPartwiseGroup(group phase4.Group) (ast.ProofPartwiseGroup, bool) {
	if !startsWithSections(group, ast.LowerPartwiseName) {
		return ast.ProofPartwiseGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections := group.Sections
	if len(sections) == 0 || sections[0].Name != ast.LowerPartwiseName {
		return ast.ProofPartwiseGroup{}, false
	}
	partwise := *p.toProofPartwiseSection(sections[0])
	parts := make([]ast.ProofPartSection, 0)
	i := 1
	for i < len(sections) {
		section := sections[i]
		i++
		if section.Name != ast.LowerPartName {
			p.tracker.Append(p.newError(fmt.Sprintf("Expected section '%s' but found '%s'",
				ast.LowerPartName, section.Name), section.MetaData.Start))
			return ast.ProofPartwiseGroup{}, false
		}
		parts = append(parts, *p.toProofPartSection(section))
	}
	return ast.ProofPartwiseGroup{
		Label:          label,
		Partwise:       partwise,
		Parts:          parts,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofPartwiseSection(section phase4.Section) *ast.ProofPartwiseSection {
	p.verifyNoArgs(section)
	return &ast.ProofPartwiseSection{CommonMetaData: toCommonMetaData(section.MetaData)}
}

func (p *parser) toProofPartSection(section phase4.Section) *ast.ProofPartSection {
	return &ast.ProofPartSection{
		Part:           p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

//////////////////////////////////// equivalently //////////////////////////////////////////////////

func (p *parser) toProofEquivalentlyGroup(group phase4.Group) (ast.ProofEquivalentlyGroup, bool) {
	if !startsWithSections(group, ast.LowerEquivalentlyName) {
		return ast.ProofEquivalentlyGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.EquivalentlySections...)
	if !ok {
		return ast.ProofEquivalentlyGroup{}, false
	}
	return ast.ProofEquivalentlyGroup{
		Label:          label,
		Equivalently:   *p.toProofEquivalentlySection(sections[ast.LowerEquivalentlyName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofEquivalentlySection(section phase4.Section) *ast.ProofEquivalentlySection {
	return &ast.ProofEquivalentlySection{
		Items:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////// allOf ////////////////////////////////////////////////////

func (p *parser) toProofAllOfGroup(group phase4.Group) (ast.ProofAllOfGroup, bool) {
	if !startsWithSections(group, ast.LowerAllOfName) {
		return ast.ProofAllOfGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.AllOfSections...)
	if !ok {
		return ast.ProofAllOfGroup{}, false
	}
	return ast.ProofAllOfGroup{
		Label:          label,
		AllOf:          *p.toProofAllOfSection(sections[ast.LowerAllOfName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofAllOfSection(section phase4.Section) *ast.ProofAllOfSection {
	return &ast.ProofAllOfSection{
		Items:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////// not //////////////////////////////////////////////////////

func (p *parser) toProofNotGroup(group phase4.Group) (ast.ProofNotGroup, bool) {
	if !startsWithSections(group, ast.LowerNotName) {
		return ast.ProofNotGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.NotSections...)
	if !ok {
		return ast.ProofNotGroup{}, false
	}
	return ast.ProofNotGroup{
		Label:          label,
		Not:            *p.toProofNotSection(sections[ast.LowerNotName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofNotSection(section phase4.Section) *ast.ProofNotSection {
	return &ast.ProofNotSection{
		Item:           p.exactlyOneProofItem(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

////////////////////////////////////////// anyOf ///////////////////////////////////////////////////

func (p *parser) toProofAnyOfGroup(group phase4.Group) (ast.ProofAnyOfGroup, bool) {
	if !startsWithSections(group, ast.LowerAnyOfName) {
		return ast.ProofAnyOfGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.AnyOfSections...)
	if !ok {
		return ast.ProofAnyOfGroup{}, false
	}
	return ast.ProofAnyOfGroup{
		Label:          label,
		AnyOf:          *p.toProofAnyOfSection(sections[ast.LowerAnyOfName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofAnyOfSection(section phase4.Section) *ast.ProofAnyOfSection {
	return &ast.ProofAnyOfSection{
		Items:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////////// oneOf ////////////////////////////////////////////////

func (p *parser) toProofOneOfGroup(group phase4.Group) (ast.ProofOneOfGroup, bool) {
	if !startsWithSections(group, ast.LowerOneOfName) {
		return ast.ProofOneOfGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.OneOfSections...)
	if !ok {
		return ast.ProofOneOfGroup{}, false
	}
	return ast.ProofOneOfGroup{
		Label:          label,
		OneOf:          *p.toProofOneOfSection(sections[ast.LowerOneOfName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofOneOfSection(section phase4.Section) *ast.ProofOneOfSection {
	return &ast.ProofOneOfSection{
		Items:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////// exists ///////////////////////////////////////////////////////

func (p *parser) toProofExistsGroup(group phase4.Group) (ast.ProofExistsGroup, bool) {
	if !startsWithSections(group, ast.LowerExistsName) {
		return ast.ProofExistsGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ExistsSections...)
	if !ok {
		return ast.ProofExistsGroup{}, false
	}
	exists := *p.toExistsSection(sections[ast.LowerExistsName])
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sect, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sect)
	}
	var suchThat *ast.ProofSuchThatSection
	if sect, ok := sections[ast.LowerSuchThatName]; ok {
		suchThat = p.toProofSuchThatSection(sect)
	}
	return ast.ProofExistsGroup{
		Label:          label,
		Exists:         exists,
		Using:          using,
		Where:          where,
		SuchThat:       suchThat,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofSuchThatSection(section phase4.Section) *ast.ProofSuchThatSection {
	return &ast.ProofSuchThatSection{
		Items:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

/////////////////////////////////////// existsUnique ///////////////////////////////////////////////

func (p *parser) toProofExistsUniqueGroup(group phase4.Group) (ast.ProofExistsUniqueGroup, bool) {
	if !startsWithSections(group, ast.LowerExistsUniqueName) {
		return ast.ProofExistsUniqueGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ExistsUniqueSections...)
	if !ok {
		return ast.ProofExistsUniqueGroup{}, false
	}
	existsUnique := *p.toExistsUniqueSection(sections[ast.LowerExistsUniqueName])
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sect, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sect)
	}
	suchThat := *p.toProofSuchThatSection(sections[ast.LowerSuchThatName])
	return ast.ProofExistsUniqueGroup{
		Label:          label,
		ExistsUnique:   existsUnique,
		Using:          using,
		Where:          where,
		SuchThat:       suchThat,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

///////////////////////////////////////// forAll ///////////////////////////////////////////////////

func (p *parser) toProofForAllGroup(group phase4.Group) (ast.ProofForAllGroup, bool) {
	if !startsWithSections(group, ast.LowerForAllName) {
		return ast.ProofForAllGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.ForAllSections...)
	if !ok {
		return ast.ProofForAllGroup{}, false
	}
	forAll := *p.toForAllSection(sections[ast.LowerForAllName])
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sec, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sec)
	}
	var suchThat *ast.ProofSuchThatSection
	if sec, ok := sections[ast.LowerSuchThatName]; ok {
		suchThat = p.toProofSuchThatSection(sec)
	}
	then := *p.toProofThenSection(sections[ast.LowerThenName])
	return ast.ProofForAllGroup{
		Label:          label,
		ForAll:         forAll,
		Using:          using,
		Where:          where,
		SuchThat:       suchThat,
		Then:           then,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofThenSection(section phase4.Section) *ast.ProofThenSection {
	return &ast.ProofThenSection{
		Then:           p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

//////////////////////////////////////////// if ////////////////////////////////////////////////////

func (p *parser) toProofIfGroup(group phase4.Group) (ast.ProofIfGroup, bool) {
	if !startsWithSections(group, ast.LowerIfName) {
		return ast.ProofIfGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.IfSections...)
	if !ok {
		return ast.ProofIfGroup{}, false
	}
	return ast.ProofIfGroup{
		Label:          label,
		If:             *p.toProofIfSection(sections[ast.LowerIfName]),
		Then:           *p.toProofThenSection(sections[ast.LowerThenName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofIfSection(section phase4.Section) *ast.ProofIfSection {
	return &ast.ProofIfSection{
		Items:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

/////////////////////////////////////////// iff ////////////////////////////////////////////////////

func (p *parser) toProofIffGroup(group phase4.Group) (ast.ProofIffGroup, bool) {
	if !startsWithSections(group, ast.LowerIffName) {
		return ast.ProofIffGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.IffSections...)
	if !ok {
		return ast.ProofIffGroup{}, false
	}
	return ast.ProofIffGroup{
		Label:          label,
		Iff:            *p.toProofIffSection(sections[ast.LowerIffName]),
		Then:           *p.toProofThenSection(sections[ast.LowerThenName]),
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

func (p *parser) toProofIffSection(section phase4.Section) *ast.ProofIffSection {
	return &ast.ProofIffSection{
		Items:          p.oneOrMoreProofItems(section),
		CommonMetaData: toCommonMetaData(section.MetaData),
	}
}

///////////////////////////////////////// let ////////////////////////////////////////////////////

func (p *parser) toProofLetGroup(group phase4.Group) (ast.ProofLetGroup, bool) {
	if !startsWithSections(group, ast.LowerLetName) {
		return ast.ProofLetGroup{}, false
	}

	label := p.getGroupLabel(group, false)
	sections, ok := IdentifySections(p.path, group.Sections, p.tracker, ast.LetSections...)
	if !ok {
		return ast.ProofLetGroup{}, false
	}
	let := *p.toLetSection(sections[ast.LowerLetName])
	var using *ast.UsingSection
	if sect, ok := sections[ast.LowerUsingName]; ok {
		using = p.toUsingSection(sect)
	}
	var where *ast.WhereSection
	if sect, ok := sections[ast.LowerWhereName]; ok {
		where = p.toWhereSection(sect)
	}
	var suchThat *ast.ProofSuchThatSection
	if sect, ok := sections[ast.LowerSuchThatName]; ok {
		suchThat = p.toProofSuchThatSection(sect)
	}
	then := *p.toProofThenSection(sections[ast.LowerThenName])
	return ast.ProofLetGroup{
		Label:          label,
		Let:            let,
		Using:          using,
		Where:          where,
		SuchThat:       suchThat,
		Then:           then,
		CommonMetaData: toCommonMetaData(group.MetaData),
	}, true
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (p *parser) verifyNoArgs(section phase4.Section) {
	if len(section.Args) > 0 {
		p.tracker.Append(p.newError("Expected no arguments", section.MetaData.Start))
	}
}

func (p *parser) oneOrMoreClauses(section phase4.Section) []ast.ClauseKind {
	return oneOrMore(p, p.toClauses(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreProofItems(section phase4.Section) []ast.ProofItemKind {
	return oneOrMore(p, p.toProofItems(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneClause(section phase4.Section) ast.ClauseKind {
	var def ast.ClauseKind = &ast.Formulation[ast.FormulationNodeKind]{}
	return exactlyOne(p, p.toClauses(section.Args), def, section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneProofItem(section phase4.Section) ast.ProofItemKind {
	var def ast.ProofItemKind = &ast.Formulation[ast.FormulationNodeKind]{}
	return exactlyOne(p, p.toProofItems(section.Args), def, section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneFormulation(
	section phase4.Section) ast.Formulation[ast.FormulationNodeKind] {
	var def ast.Formulation[ast.FormulationNodeKind] = ast.Formulation[ast.FormulationNodeKind]{}
	return exactlyOne(p, p.toFormulations(section.Args), def, section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreFormulation(
	section phase4.Section) []ast.Formulation[ast.FormulationNodeKind] {
	return oneOrMore(p, p.toFormulations(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreSpecs(section phase4.Section) []ast.Spec {
	return oneOrMore(p, p.toSpecs(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreAliases(section phase4.Section) []ast.Alias {
	return oneOrMore(p, p.toAliases(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneAlias(section phase4.Section) ast.Alias {
	var def ast.Alias = ast.Alias{}
	return exactlyOne(p, p.toAliases(section.Args), def, section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneSpec(section phase4.Section) ast.Spec {
	var def ast.Spec = ast.Spec{}
	return exactlyOne(p, p.toSpecs(section.Args), def, section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreTargets(section phase4.Section) []ast.Target {
	return oneOrMore(p, p.toTargets(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreTextItems(section phase4.Section) []ast.TextItem {
	return oneOrMore(p, p.toTextItems(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) zeroOrMoreTextItems(section phase4.Section) []ast.TextItem {
	return p.toTextItems(section.Args)
}

func (p *parser) oneOrMorePersonKinds(section phase4.Section) []ast.PersonKind {
	return oneOrMore(p, p.toPersonKinds(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreResourceKinds(section phase4.Section) []ast.ResourceKind {
	return oneOrMore(p, p.toResourceKinds(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneTarget(section phase4.Section) ast.Target {
	var def ast.Target = ast.Target{}
	return exactlyOne(p, p.toTargets(section.Args), def, section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneSignatureItem(section phase4.Section) ast.Formulation[*ast.Signature] {
	var def ast.Formulation[*ast.Signature] = ast.Formulation[*ast.Signature]{}
	return exactlyOne(p, p.toSignaturesItems(section.Args), def, section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreSignatureItems(section phase4.Section) []ast.Formulation[*ast.Signature] {
	return oneOrMore(p, p.toSignaturesItems(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneTextItem(section phase4.Section) ast.TextItem {
	return exactlyOne(p, p.toTextItems(section.Args),
		ast.TextItem{}, section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreDocumentedKinds(section phase4.Section) []ast.DocumentedKind {
	return oneOrMore(p, p.toDocumentedKinds(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreSpecifyKinds(section phase4.Section) []ast.SpecifyKind {
	return oneOrMore(p, p.toSpecifyKinds(section.Args), section.MetaData.Start, p.tracker)
}

///////////////////////////////////// support functions ////////////////////////////////////////////

func oneOrMore[T any](p *parser, items []T, position ast.Position,
	tracker frontend.IDiagnosticTracker) []T {
	if len(items) == 0 {
		tracker.Append(p.newError("Expected at least one item", position))
		return []T{}
	}
	return items
}

func exactlyOne[T any](p *parser, items []T, defaultItem T, position ast.Position,
	tracker frontend.IDiagnosticTracker) T {
	if len(items) != 1 {
		tracker.Append(p.newError("Expected at exactly one item", position))
	}
	if len(items) == 0 {
		return defaultItem
	}
	return items[0]
}

func startsWithSections(group phase4.Group, names ...string) bool {
	if len(names) > len(group.Sections) {
		return false
	}

	for i, name := range names {
		if group.Sections[i].Name != name {
			return false
		}
	}

	return true
}

func endsWithSection(group phase4.Group, name string) bool {
	sections := group.Sections
	if len(sections) == 0 {
		return false
	} else {
		return sections[len(sections)-1].Name == name
	}
}

func (p *parser) newError(message string, position ast.Position) frontend.Diagnostic {
	return frontend.Diagnostic{
		Path:     p.path,
		Type:     frontend.Error,
		Origin:   frontend.Phase5ParserOrigin,
		Message:  message,
		Position: position,
	}
}

func sectionNamesToString(names []string) string {
	result := ""
	for i, name := range names {
		result += name + ":"
		if i != len(names)-1 {
			result += "\n"
		}
	}
	return result
}

func toCommonMetaData(metaData phase4.MetaData) ast.CommonMetaData {
	return ast.CommonMetaData{
		Start: metaData.Start,
		Key:   metaData.Key,
		Scope: nil,
	}
}
