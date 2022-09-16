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
)

func NewPhase5Parser(tracker frontend.DiagnosticTracker) parser {
	return parser{
		tracker: tracker,
	}
}

//////////////////////////////////////////////////////////////////////////

type parser struct {
	tracker frontend.DiagnosticTracker
}

/////////////////////////// allOf ///////////////////////////////////////

func (p *parser) toAllOfGroup(group phase4.Group) (ast.AllOfGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "allOf")
	if !ok {
		return ast.AllOfGroup{}, false
	}
	return ast.AllOfGroup{
		AllOf: *p.toAllOfSection(sections["allOf"]),
	}, true
}

func (p *parser) toAllOfSection(section phase4.Section) *ast.AllOfSection {
	return &ast.AllOfSection{
		Clauses: p.oneOrMoreClauses(section),
	}
}

/////////////////////////////// not //////////////////////////////////////

func (p *parser) toNotGroup(group phase4.Group) (ast.NotGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "not")
	if !ok {
		return ast.NotGroup{}, false
	}
	return ast.NotGroup{
		Not: *p.toNotSection(sections["not"]),
	}, true
}

func (p *parser) toNotSection(section phase4.Section) *ast.NotSection {
	return &ast.NotSection{
		Clause: p.exactlyOneClause(section),
	}
}

//////////////////////////////////// anyOf ////////////////////////////////

func (p *parser) toAnyOfGroup(group phase4.Group) (ast.AnyOfGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "anyOf")
	if !ok {
		return ast.AnyOfGroup{}, false
	}
	return ast.AnyOfGroup{
		AnyOf: *p.toAnyOfSection(sections["anyOf"]),
	}, true
}

func (p *parser) toAnyOfSection(section phase4.Section) *ast.AnyOfSection {
	return &ast.AnyOfSection{
		Clauses: p.oneOrMoreClauses(section),
	}
}

////////////////////////////////////// oneOf /////////////////////////////

func (p *parser) toOneOfGroup(group phase4.Group) (ast.OneOfGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "oneOf")
	if !ok {
		return ast.OneOfGroup{}, false
	}
	return ast.OneOfGroup{
		OneOf: *p.toOneOfSection(sections["oneOf"]),
	}, true
}

func (p *parser) toOneOfSection(section phase4.Section) *ast.OneOfSection {
	return &ast.OneOfSection{
		Clauses: p.oneOrMoreClauses(section),
	}
}

/////////////////////////////// exists //////////////////////////////////

func (p *parser) ToExistsGroup(group phase4.Group) (ast.ExistsGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"exists",
		"where?",
		"suchThat?")
	if !ok {
		return ast.ExistsGroup{}, false
	}
	exists := *p.toExistsSection(sections["exists"])
	var where *ast.WhereSection
	if sect, ok := sections["where"]; ok {
		where = p.toWhereSection(sect)
	}
	var suchThat *ast.SuchThatSection
	if sect, ok := sections["suchThat"]; ok {
		suchThat = p.toSuchThatSection(sect)
	}
	return ast.ExistsGroup{
		Exists:   exists,
		Where:    where,
		SuchThat: suchThat,
	}, true
}

func (p *parser) toExistsSection(section phase4.Section) *ast.ExistsSection {
	return &ast.ExistsSection{
		Targets: p.oneOrMoreTargets(section),
	}
}

func (p *parser) toWhereSection(section phase4.Section) *ast.WhereSection {
	return &ast.WhereSection{
		Specs: p.oneOrMoreSpecs(section),
	}
}

func (p *parser) toSuchThatSection(section phase4.Section) *ast.SuchThatSection {
	return &ast.SuchThatSection{
		Clauses: p.oneOrMoreClauses(section),
	}
}

/////////////////////////////// existsUnique //////////////////////////////////

func (p *parser) ToExistsUniqueGroup(group phase4.Group) (ast.ExistsUniqueGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"existsUnique",
		"where?",
		"suchThat?")
	if !ok {
		return ast.ExistsUniqueGroup{}, false
	}
	existsUnique := *p.toExistsUniqueSection(sections["existsUnique"])
	var where *ast.WhereSection
	if sect, ok := sections["where"]; ok {
		where = p.toWhereSection(sect)
	}
	var suchThat *ast.SuchThatSection
	if sect, ok := sections["suchThat"]; ok {
		suchThat = p.toSuchThatSection(sect)
	}
	return ast.ExistsUniqueGroup{
		ExistsUnique: existsUnique,
		Where:        where,
		SuchThat:     suchThat,
	}, true
}

func (p *parser) toExistsUniqueSection(section phase4.Section) *ast.ExistsUniqueSection {
	return &ast.ExistsUniqueSection{
		Targets: p.oneOrMoreTargets(section),
	}
}

///////////////////////// forAll /////////////////////////////////////////

func (p *parser) ToForAllGroup(group phase4.Group) (ast.ForAllGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"forAll",
		"where?",
		"suchThat?",
		"then")
	if !ok {
		return ast.ForAllGroup{}, false
	}
	forAll := *p.toForAllSection(sections["forAll"])
	var where *ast.WhereSection
	if sec, ok := sections["where"]; ok {
		where = p.toWhereSection(sec)
	}
	var suchThat *ast.SuchThatSection
	if sec, ok := sections["suchThat"]; ok {
		suchThat = p.toSuchThatSection(sec)
	}
	then := *p.toThenSection(sections["then"])
	return ast.ForAllGroup{
		ForAll:   forAll,
		Where:    where,
		SuchThat: suchThat,
		Then:     then,
	}, true
}

func (p *parser) toForAllSection(section phase4.Section) *ast.ForAllSection {
	return &ast.ForAllSection{
		Targets: p.oneOrMoreTargets(section),
	}
}

func (p *parser) toThenSection(section phase4.Section) *ast.ThenSection {
	return &ast.ThenSection{
		Clauses: p.oneOrMoreClauses(section),
	}
}

/////////////////////////////// if ///////////////////////////////////////

func (p *parser) ToIfGroup(group phase4.Group) (ast.IfGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"if",
		"then")
	if !ok {
		return ast.IfGroup{}, false
	}
	return ast.IfGroup{
		If:   *p.toIfSection(sections["if"]),
		Then: *p.toThenSection(sections["then"]),
	}, true
}

func (p *parser) toIfSection(section phase4.Section) *ast.IfSection {
	return &ast.IfSection{
		Clauses: p.oneOrMoreClauses(section),
	}
}

/////////////////////////////// iff ///////////////////////////////////////

func (p *parser) ToIffGroup(group phase4.Group) (ast.IffGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"iff",
		"then")
	if !ok {
		return ast.IffGroup{}, false
	}
	return ast.IffGroup{
		Iff:  *p.toIffSection(sections["iff"]),
		Then: *p.toThenSection(sections["then"]),
	}, true
}

func (p *parser) toIffSection(section phase4.Section) *ast.IffSection {
	return &ast.IffSection{
		Clauses: p.oneOrMoreClauses(section),
	}
}

/////////////////////////// generated ////////////////////////////////////

func (p *parser) ToGeneratedGroup(group phase4.Group) (ast.GeneratedGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"generated",
		"from",
		"when?")
	if !ok {
		return ast.GeneratedGroup{}, false
	}
	generated := *p.toGeneratedSection(sections["generated"])
	from := *p.toFromSection(sections["from"])
	var when *ast.WhenSection
	if sec, ok := sections["when"]; ok {
		when = p.toWhenSection(sec)
	}
	return ast.GeneratedGroup{
		Generated: generated,
		From:      from,
		When:      when,
	}, true
}

func (p *parser) toGeneratedSection(section phase4.Section) *ast.GeneratedSection {
	p.verifyNoArgs(section)
	return &ast.GeneratedSection{}
}

func (p *parser) toFromSection(section phase4.Section) *ast.FromSection {
	return &ast.FromSection{
		Items: p.oneOrMoreTargets(section),
	}
}

func (p *parser) toWhenSection(section phase4.Section) *ast.WhenSection {
	return &ast.WhenSection{
		Items: p.oneOrMoreClauses(section),
	}
}

///////////////////////////// piecewise /////////////////////////////////

func (p *parser) ToPiecewiseGroup(group phase4.Group) (ast.PiecewiseGroup, bool) {
	sections := group.Sections
	if len(sections) == 0 || sections[0].Name != "piecewise" {
		return ast.PiecewiseGroup{}, false
	}
	piecewise := *p.toPiecewiseSection(sections[0])
	ifThens := make([]ast.IfThen, 0)
	i := 1
	for i < len(sections) {
		section1 := sections[i]
		if section1.Name == "else" {
			break
		}
		i++
		if section1.Name != "if" {
			p.tracker.Append(newError(fmt.Sprintf("Expected section 'if' but found '%s'", section1.Name), section1.MetaData.Start))
			return ast.PiecewiseGroup{}, false
		}
		if i >= len(sections) {
			p.tracker.Append(newError("Expected section 'then' to follow an 'if' section", section1.MetaData.Start))
			return ast.PiecewiseGroup{}, false
		}
		section2 := sections[i]
		i++
		if section2.Name != "else" {
			p.tracker.Append(newError(fmt.Sprintf("Expected section 'then' but found '%s'", section2.Name), section2.MetaData.Start))
			return ast.PiecewiseGroup{}, false
		}
		ifThens = append(ifThens, ast.IfThen{
			If:   *p.toIfSection(section1),
			Then: *p.toThenSection(section2),
		})
	}
	var elseSec *ast.ElseSection
	if i < len(sections) && sections[i].Name == "else" {
		elseSec = p.toElseSection(sections[i])
		i++
	}
	invalid := i < len(sections)
	for i < len(sections) {
		sec := sections[i]
		i++
		p.tracker.Append(newError(fmt.Sprintf("Unexpected section '%s'", sec.Name), sec.MetaData.Start))
	}
	if invalid {
		return ast.PiecewiseGroup{}, false
	}
	return ast.PiecewiseGroup{
		Piecewise: piecewise,
		IfThen:    ifThens,
		Else:      elseSec,
	}, true
}

func (p *parser) toPiecewiseSection(section phase4.Section) *ast.PiecewiseSection {
	p.verifyNoArgs(section)
	return &ast.PiecewiseSection{}
}

func (p *parser) toElseSection(section phase4.Section) *ast.ElseSection {
	return &ast.ElseSection{
		Items: p.oneOrMoreClauses(section),
	}
}

/////////////////////////////// as via ///////////////////////////////////

func (p *parser) ToAsViaGroup(group phase4.Group) (ast.AsViaGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"as",
		"via")
	if !ok {
		return ast.AsViaGroup{}, false
	}
	return ast.AsViaGroup{
		As:  *p.toAsSection(sections["as"]),
		Via: *p.toViaSection(sections["via"]),
	}, true
}

func (p *parser) toAsSection(section phase4.Section) *ast.AsSection {
	return &ast.AsSection{
		As: p.exactlyOneSignatureItem(section),
	}
}

func (p *parser) toViaSection(section phase4.Section) *ast.ViaSection {
	return &ast.ViaSection{
		Via: p.exactlyOneClause(section),
	}
}

////////////////////////// as through ////////////////////////////////////

func (p *parser) ToAsThroughGroup(group phase4.Group) (ast.AsThroughGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"as",
		"through",
		"as?",
		"states?")
	if !ok {
		return ast.AsThroughGroup{}, false
	}
	as := *p.toAsSection(sections["as"])
	through := *p.toThroughSection(sections["through"])
	var throughAs *ast.AsSection
	if sec, ok := sections["as1"]; ok {
		throughAs = p.toAsSection(sec)
	}
	var states *ast.AsStatesSection
	if sec, ok := sections["states"]; ok {
		states = p.toAsStatesSection(sec)
	}
	return ast.AsThroughGroup{
		As:        as,
		Through:   through,
		ThroughAs: throughAs,
		States:    states,
	}, true
}

func (p *parser) toThroughSection(section phase4.Section) *ast.ThroughSection {
	return &ast.ThroughSection{
		Through: p.exactlyOneSpec(section),
	}
}

func (p *parser) toAsStatesSection(section phase4.Section) *ast.AsStatesSection {
	return &ast.AsStatesSection{
		As: p.exactlyOneSignatureItem(section),
	}
}

////////////////////////////// viewable type ///////////////////////////

func (p *parser) toViewableSection(section phase4.Section) *ast.ViewableSection {
	return nil
}

func (p *parser) ToViewableType(group phase4.Group) (ast.ViewableType, bool) {
	if grp, ok := p.ToAsViaGroup(group); ok {
		return grp, true
	}

	if grp, ok := p.ToAsThroughGroup(group); ok {
		return grp, true
	}

	return nil, false
}

////////////////////// infix, prefix, postfix, symbol ///////////////////

func (p *parser) toProvidesStatesSection(section phase4.Section) *ast.ProvidesStatesSection {
	return &ast.ProvidesStatesSection{
		Clause: p.exactlyOneClause(section),
	}
}

func (p *parser) toProvidesDefinesSection(section phase4.Section) *ast.ProvidesDefinesSection {
	return &ast.ProvidesDefinesSection{
		Clause: p.exactlyOneClause(section),
	}
}

func (p *parser) toProvidesStatesDefinesType(section phase4.Section) (ast.ProvidesStatesDefinesType, bool) {
	if section.Name == "states" {
		return *p.toProvidesStatesSection(section), true
	} else if section.Name == "defines" {
		return *p.toProvidesDefinesSection(section), true
	} else {
		return nil, false
	}
}

func (p *parser) toInfixSection(section phase4.Section) *ast.InfixSection {
	return &ast.InfixSection{
		Infix: p.oneOrMoreTargets(section),
	}
}

func (p *parser) toPrefixSection(section phase4.Section) *ast.PrefixSection {
	return &ast.PrefixSection{
		Prefix: p.oneOrMoreTargets(section),
	}
}

func (p *parser) toPostfixSection(section phase4.Section) *ast.PostfixSection {
	return &ast.PostfixSection{
		Postfix: p.oneOrMoreTargets(section),
	}
}

func (p *parser) toSymbolSection(section phase4.Section) *ast.SymbolSection {
	return &ast.SymbolSection{
		Symbol: p.oneOrMoreTargets(section),
	}
}

func (p *parser) toInfixGroup(group phase4.Group) (ast.InfixGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"infix",
		"when?",
		"states")
	if ok {
		infix := *p.toInfixSection(sections["infix"])
		var when *ast.WhenSection
		if sec, ok := sections["when"]; ok {
			when = p.toWhenSection(sec)
		}
		states := *p.toProvidesStatesSection(sections["states"])
		return ast.InfixGroup{
			Infix:         infix,
			When:          when,
			StatesDefines: states,
		}, true
	}

	sections, ok = IdentifySections(group.Sections, p.tracker,
		"infix",
		"when?",
		"defines")
	if ok {
		infix := *p.toInfixSection(sections["infix"])
		var when *ast.WhenSection
		if sec, ok := sections["when"]; ok {
			when = p.toWhenSection(sec)
		}
		defines := *p.toProvidesDefinesSection(sections["defines"])
		return ast.InfixGroup{
			Infix:         infix,
			When:          when,
			StatesDefines: defines,
		}, true
	}

	return ast.InfixGroup{}, false
}

func (p *parser) toPrefixGroup(group phase4.Group) (ast.PrefixGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"prefix",
		"when?",
		"states")
	if ok {
		prefix := *p.toPrefixSection(sections["prefix"])
		var when *ast.WhenSection
		if sec, ok := sections["when"]; ok {
			when = p.toWhenSection(sec)
		}
		states := *p.toProvidesStatesSection(sections["states"])
		return ast.PrefixGroup{
			Prefix:        prefix,
			When:          when,
			StatesDefines: states,
		}, true
	}

	sections, ok = IdentifySections(group.Sections, p.tracker,
		"prefix",
		"when?",
		"defines")
	if ok {
		prefix := *p.toPrefixSection(sections["prefix"])
		var when *ast.WhenSection
		if sec, ok := sections["when"]; ok {
			when = p.toWhenSection(sec)
		}
		defines := *p.toProvidesDefinesSection(sections["defines"])
		return ast.PrefixGroup{
			Prefix:        prefix,
			When:          when,
			StatesDefines: defines,
		}, true
	}

	return ast.PrefixGroup{}, false
}

func (p *parser) toPostfixGroup(group phase4.Group) (ast.PostfixGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"postfix",
		"when?",
		"states")
	if ok {
		postfix := *p.toPostfixSection(sections["postfix"])
		var when *ast.WhenSection
		if sec, ok := sections["when"]; ok {
			when = p.toWhenSection(sec)
		}
		states := *p.toProvidesStatesSection(sections["states"])
		return ast.PostfixGroup{
			Postfix:       postfix,
			When:          when,
			StatesDefines: states,
		}, true
	}

	sections, ok = IdentifySections(group.Sections, p.tracker,
		"postfix",
		"when?",
		"defines")
	if ok {
		postfix := *p.toPostfixSection(sections["postfix"])
		var when *ast.WhenSection
		if sec, ok := sections["when"]; ok {
			when = p.toWhenSection(sec)
		}
		defines := *p.toProvidesDefinesSection(sections["defines"])
		return ast.PostfixGroup{
			Postfix:       postfix,
			When:          when,
			StatesDefines: defines,
		}, true
	}

	return ast.PostfixGroup{}, false
}

func (p *parser) toSymbolGroup(group phase4.Group) (ast.SymbolGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"symbol",
		"when?",
		"states")
	if ok {
		symbol := *p.toSymbolSection(sections["symbol"])
		var when *ast.WhenSection
		if sec, ok := sections["when"]; ok {
			when = p.toWhenSection(sec)
		}
		states := *p.toProvidesStatesSection(sections["states"])
		return ast.SymbolGroup{
			Symbol:        symbol,
			When:          when,
			StatesDefines: states,
		}, true
	}

	sections, ok = IdentifySections(group.Sections, p.tracker,
		"symbol",
		"when?",
		"defines")
	if ok {
		symbol := *p.toSymbolSection(sections["symbol"])
		var when *ast.WhenSection
		if sec, ok := sections["when"]; ok {
			when = p.toWhenSection(sec)
		}
		defines := *p.toProvidesDefinesSection(sections["defines"])
		return ast.SymbolGroup{
			Symbol:        symbol,
			When:          when,
			StatesDefines: defines,
		}, true
	}

	return ast.SymbolGroup{}, false
}

////////////////////////////// codified /////////////////////////////////

func (p *parser) toCodifiedSection(section phase4.Section) *ast.CodifiedSection {
	return &ast.CodifiedSection{
		Codified: p.oneOrMoreCodifiedType(section),
	}
}

func (p *parser) oneOrMoreCodifiedType(section phase4.Section) []ast.CodifiedType {
	return oneOrMore(p.toCodifiedTypes(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) toCodifiedTypes(args []phase4.Argument) []ast.CodifiedType {
	result := make([]ast.CodifiedType, 0)
	for _, arg := range args {
		if codeType, ok := p.toCodifiedTypeFromArg(arg); ok {
			result = append(result, codeType)
		}
	}
	return result
}

func (p *parser) toCodifiedTypeFromArg(arg phase4.Argument) (ast.CodifiedType, bool) {
	group, ok := arg.Arg.(phase4.Group)
	if !ok {
		return nil, false
	}
	return p.toCodifiedTypeFromGroup(group)
}

func (p *parser) toCodifiedTypeFromGroup(group phase4.Group) (ast.CodifiedType, bool) {
	if grp, ok := p.toWrittenGroup(group); ok {
		return grp, true
	} else if grp, ok := p.toCalledGroup(group); ok {
		return grp, true
	} else if grp, ok := p.toWritingGroup(group); ok {
		return grp, true
	} else {
		return nil, false
	}
}

func (p *parser) toWrittenGroup(group phase4.Group) (ast.WrittenGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "written")
	if !ok {
		return ast.WrittenGroup{}, false
	}
	return ast.WrittenGroup{
		Written: *p.toWrittenSection(sections["written"]),
	}, true
}

func (p *parser) toWrittenSection(section phase4.Section) *ast.WrittenSection {
	return &ast.WrittenSection{
		Written: p.oneOrMoreTextItems(section),
	}
}

func (p *parser) toCalledGroup(group phase4.Group) (ast.CalledGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "called")
	if !ok {
		return ast.CalledGroup{}, false
	}
	return ast.CalledGroup{
		Called: *p.toCalledSection(sections["called"]),
	}, true
}

func (p *parser) toCalledSection(section phase4.Section) *ast.CalledSection {
	return &ast.CalledSection{
		Called: p.oneOrMoreTextItems(section),
	}
}

func (p *parser) toWritingGroup(group phase4.Group) (ast.WritingGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"writing",
		"as")
	if !ok {
		return ast.WritingGroup{}, false
	}
	writing := *p.toWritingSection(sections["writing"])
	as := *p.toWritingAsSection(sections["as"])
	return ast.WritingGroup{
		Writing: writing,
		As:      as,
	}, true
}

func (p *parser) toWritingSection(section phase4.Section) *ast.WritingSection {
	return &ast.WritingSection{
		Writing: p.oneOrMoreTargets(section),
	}
}

func (p *parser) toWritingAsSection(section phase4.Section) *ast.WritingAsSection {
	return &ast.WritingAsSection{
		As: p.oneOrMoreTextItems(section),
	}
}

////////////////////////// documented ////////////////////////////////////

func (p *parser) toDocumentedSection(section phase4.Section) *ast.DocumentedSection {
	return &ast.DocumentedSection{
		Documented: p.oneOrMoreDocumentedTypes(section),
	}
}

func (p *parser) toDocumentedType(arg phase4.Argument) (ast.DocumentedType, bool) {
	switch group := arg.Arg.(type) {
	case phase4.Group:
		if grp, ok := p.toLooselyGroup(group); ok {
			return grp, true
		} else if grp, ok := p.toOverviewGroup(group); ok {
			return grp, true
		} else if grp, ok := p.toMotivationGroup(group); ok {
			return grp, true
		} else if grp, ok := p.toHistoryGroup(group); ok {
			return grp, true
		} else if grp, ok := p.toExamplesGroup(group); ok {
			return grp, true
		} else if grp, ok := p.toRelatedGroup(group); ok {
			return grp, true
		} else if grp, ok := p.toDiscoveredGroup(group); ok {
			return grp, true
		} else if grp, ok := p.toNotesGroup(group); ok {
			return grp, true
		}
	}

	p.tracker.Append(newError("Expected a documented item", arg.MetaData.Start))
	return nil, false
}

func (p *parser) toLooselyGroup(group phase4.Group) (ast.LooselyGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "loosely")
	if !ok {
		return ast.LooselyGroup{}, false
	}
	return ast.LooselyGroup{
		Loosely: *p.toLooselySection(sections["loosely"]),
	}, true
}

func (p *parser) toLooselySection(section phase4.Section) *ast.LooselySection {
	return &ast.LooselySection{
		Loosely: p.exactlyOneTextItem(section),
	}
}

func (p *parser) toOverviewGroup(group phase4.Group) (ast.OverviewGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "overview")
	if !ok {
		return ast.OverviewGroup{}, false
	}
	return ast.OverviewGroup{
		Overview: *p.toOverviewSection(sections["overview"]),
	}, true
}

func (p *parser) toOverviewSection(section phase4.Section) *ast.OverviewSection {
	return &ast.OverviewSection{
		Overview: p.exactlyOneTextItem(section),
	}
}

func (p *parser) toMotivationGroup(group phase4.Group) (ast.MotivationGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "motivation")
	if !ok {
		return ast.MotivationGroup{}, false
	}
	return ast.MotivationGroup{
		Motivation: *p.toMotivationSection(sections["motivation"]),
	}, true
}

func (p *parser) toMotivationSection(section phase4.Section) *ast.MotivationSection {
	return &ast.MotivationSection{
		Motivation: p.exactlyOneTextItem(section),
	}
}

func (p *parser) toHistoryGroup(group phase4.Group) (ast.HistoryGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "history")
	if !ok {
		return ast.HistoryGroup{}, false
	}
	return ast.HistoryGroup{
		History: *p.toHistorySection(sections["history"]),
	}, true
}

func (p *parser) toHistorySection(section phase4.Section) *ast.HistorySection {
	return &ast.HistorySection{
		History: p.exactlyOneTextItem(section),
	}
}

func (p *parser) toExamplesGroup(group phase4.Group) (ast.ExamplesGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "examples")
	if !ok {
		return ast.ExamplesGroup{}, false
	}
	return ast.ExamplesGroup{
		Examples: *p.toExamplesSection(sections["examples"]),
	}, true
}

func (p *parser) toExamplesSection(section phase4.Section) *ast.ExamplesSection {
	return &ast.ExamplesSection{
		Examples: p.oneOrMoreTextItems(section),
	}
}

func (p *parser) toRelatedGroup(group phase4.Group) (ast.RelatedGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "related")
	if !ok {
		return ast.RelatedGroup{}, false
	}
	return ast.RelatedGroup{
		Related: *p.toRelatedSection(sections["related"]),
	}, true
}

func (p *parser) toRelatedSection(section phase4.Section) *ast.RelatedSection {
	return &ast.RelatedSection{
		Related: p.oneOrMoreSignatureItems(section),
	}
}

func (p *parser) toDiscoveredGroup(group phase4.Group) (ast.DiscoveredGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "discovered")
	if !ok {
		return ast.DiscoveredGroup{}, false
	}
	return ast.DiscoveredGroup{
		Discovered: *p.toDiscoveredSection(sections["discovered"]),
	}, true
}

func (p *parser) toDiscoveredSection(section phase4.Section) *ast.DiscoveredSection {
	return &ast.DiscoveredSection{
		Discovered: p.oneOrMoreTextItems(section),
	}
}

func (p *parser) toNotesGroup(group phase4.Group) (ast.NotesGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "notes")
	if !ok {
		return ast.NotesGroup{}, false
	}
	return ast.NotesGroup{
		Notes: *p.toNotesSection(sections["notes"]),
	}, true
}

func (p *parser) toNotesSection(section phase4.Section) *ast.NotesSection {
	return &ast.NotesSection{
		Notes: p.oneOrMoreTextItems(section),
	}
}

/////////////////////////// viewable /////////////////////////////////////

// TODO: finish this

////////////////////////// provides //////////////////////////////////////

func (p *parser) toProvidesSection(section phase4.Section) *ast.ProvidesSection {
	return &ast.ProvidesSection{
		Provides: p.oneOrMoreProvidesType(section),
	}
}

func (p *parser) oneOrMoreProvidesType(section phase4.Section) []ast.ProvidesType {
	return oneOrMore(p.toProvidesTypes(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) toProvidesTypes(args []phase4.Argument) []ast.ProvidesType {
	result := make([]ast.ProvidesType, 0)
	for _, arg := range args {
		if providesType, ok := p.toProvidesTypeFromArg(arg); ok {
			result = append(result, providesType)
		}
	}
	return result
}

func (p *parser) toProvidesTypeFromArg(arg phase4.Argument) (ast.ProvidesType, bool) {
	group, ok := arg.Arg.(phase4.Group)
	if !ok {
		return nil, false
	}
	return p.toProvidesTypeFromGroup(group)
}

func (p *parser) toProvidesTypeFromGroup(group phase4.Group) (ast.ProvidesType, bool) {
	if grp, ok := p.toInfixGroup(group); ok {
		return grp, true
	} else if grp, ok := p.toPrefixGroup(group); ok {
		return grp, true
	} else if grp, ok := p.toPostfixGroup(group); ok {
		return grp, true
	} else if grp, ok := p.toSymbolGroup(group); ok {
		return grp, true
	} else {
		return nil, false
	}
}

////////////////////////// using ////////////////////////////////////////

func (p *parser) toUsingSection(section phase4.Section) *ast.UsingSection {
	return &ast.UsingSection{
		Using: p.oneOrMoreClauses(section),
	}
}

////////////////////////// justified ////////////////////////////////////

func (p *parser) toJustifiedSection(section phase4.Section) *ast.JustifiedSection {
	return &ast.JustifiedSection{
		Justified: p.oneOrMoreJustifiedType(section),
	}
}

func (p *parser) oneOrMoreJustifiedType(section phase4.Section) []ast.JustifiedType {
	return oneOrMore(p.toJustifiedTypes(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) toJustifiedTypes(args []phase4.Argument) []ast.JustifiedType {
	result := make([]ast.JustifiedType, 0)
	for _, arg := range args {
		if justifiedType, ok := p.toJustifiedTypeFromArg(arg); ok {
			result = append(result, justifiedType)
		}
	}
	return result
}

func (p *parser) toJustifiedTypeFromArg(arg phase4.Argument) (ast.JustifiedType, bool) {
	group, ok := arg.Arg.(phase4.Group)
	if !ok {
		return nil, false
	}
	return p.toJustifiedTypeFromGroup(group)
}

func (p *parser) toJustifiedTypeFromGroup(group phase4.Group) (ast.JustifiedType, bool) {
	if grp, ok := p.toLabelGroup(group); ok {
		return grp, true
	} else if grp, ok := p.toByGroup(group); ok {
		return grp, true
	} else {
		return nil, false
	}
}

func (p *parser) toLabelSection(section phase4.Section) *ast.LabelSection {
	return &ast.LabelSection{
		Label: p.exactlyOneTextItem(section),
	}
}

func (p *parser) toLabelGroup(group phase4.Group) (ast.LabelGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"label",
		"by")
	if !ok {
		return ast.LabelGroup{}, false
	}
	return ast.LabelGroup{
		Label: *p.toLabelSection(sections["label"]),
		By:    *p.toBySection(sections["by"]),
	}, true
}

func (p *parser) toByGroup(group phase4.Group) (ast.ByGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "by")
	if !ok {
		return ast.ByGroup{}, false
	}
	return ast.ByGroup{
		By: *p.toBySection(sections["by"]),
	}, true
}

func (p *parser) toBySection(section phase4.Section) *ast.BySection {
	return &ast.BySection{
		By: p.oneOrMoreTextItems(section),
	}
}

////////////////////////// references ///////////////////////////////////

func (p *parser) toReferencesSection(section phase4.Section) *ast.ReferencesSection {
	return &ast.ReferencesSection{
		References: p.oneOrMoreTextItems(section),
	}
}

///////////////////////// metadata //////////////////////////////////////

func (p *parser) toMetadataSection(section phase4.Section) *ast.MetadataSection {
	return nil
}

// TODO: finish this

//////////////////////// describes //////////////////////////////////////

func (p *parser) toDescribesGroup(group phase4.Group) (ast.DescribesGroup, bool) {
	id := p.getId(group, true)
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"Describes",
		"with?",
		"given?",
		"when?",
		"suchThat?",
		"extends?",
		"satisfies?",
		"Viewable?",
		"Provides?",
		"Using?",
		"Codified",
		"Documented?",
		"Justified?",
		"References?",
		"Metadata?")
	if !ok || id == nil {
		return ast.DescribesGroup{}, false
	}
	describes := *p.toDescribesSection(sections["Describes"])
	var with *ast.WithSection
	if sec, ok := sections["with"]; ok {
		with = p.toWithSection(sec)
	}
	var given *ast.GivenSection
	if sec, ok := sections["given"]; ok {
		given = p.toGivenSection(sec)
	}
	var when *ast.WhenSection
	if sec, ok := sections["when"]; ok {
		when = p.toWhenSection(sec)
	}
	var suchThat *ast.SuchThatSection
	if sec, ok := sections["suchThat"]; ok {
		suchThat = p.toSuchThatSection(sec)
	}
	var extends *ast.ExtendsSection
	if sec, ok := sections["extends"]; ok {
		extends = p.toExtendsSection(sec)
	}
	var satisfies *ast.SatisfiesSection
	if sec, ok := sections["satisfies"]; ok {
		satisfies = p.toSatisfiesSection(sec)
	}
	var viewable *ast.ViewableSection
	if sec, ok := sections["Viewable"]; ok {
		viewable = p.toViewableSection(sec)
	}
	var provides *ast.ProvidesSection
	if sec, ok := sections["Provides"]; ok {
		provides = p.toProvidesSection(sec)
	}
	var using *ast.UsingSection
	if sec, ok := sections["Using"]; ok {
		using = p.toUsingSection(sec)
	}
	codified := *p.toCodifiedSection(sections["Codified"])
	var documented *ast.DocumentedSection
	if sec, ok := sections["Documented"]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var justified *ast.JustifiedSection
	if sec, ok := sections["Justified"]; ok {
		justified = p.toJustifiedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections["Justified"]; ok {
		references = p.toReferencesSection(sec)
	}
	var metadata *ast.MetadataSection
	if sec, ok := sections["Metadata"]; ok {
		metadata = p.toMetadataSection(sec)
	}
	return ast.DescribesGroup{
		Id:         *id,
		Describes:  describes,
		With:       with,
		Given:      given,
		When:       when,
		SuchThat:   suchThat,
		Extends:    extends,
		Satisfies:  satisfies,
		Viewable:   viewable,
		Provides:   provides,
		Using:      using,
		Codified:   codified,
		Documented: documented,
		Justified:  justified,
		References: references,
		Metadata:   metadata,
	}, true
}

func (p *parser) toDescribesSection(section phase4.Section) *ast.DescribesSection {
	return &ast.DescribesSection{
		Describes: p.exactlyOneTarget(section),
	}
}

func (p *parser) toExtendsSection(section phase4.Section) *ast.ExtendsSection {
	return &ast.ExtendsSection{
		Extends: p.oneOrMoreClauses(section),
	}
}

func (p *parser) toSatisfiesSection(section phase4.Section) *ast.SatisfiesSection {
	return &ast.SatisfiesSection{
		Satisfies: p.oneOrMoreClauses(section),
	}
}

/////////////////////// declares ////////////////////////////////////////

func (p *parser) toDeclaresGroup(group phase4.Group) (ast.DeclaresGroup, bool) {
	id := p.getId(group, true)
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"Declares",
		"with?",
		"given?",
		"when?",
		"suchThat?",
		"means?",
		"defines?",
		"Viewable?",
		"Provides?",
		"Using?",
		"Codified",
		"Documented?",
		"Justified?",
		"References?",
		"Metadata?")
	if !ok || id == nil {
		return ast.DeclaresGroup{}, false
	}
	declares := *p.toDeclaresSection(sections["Declares"])
	var with *ast.WithSection
	if sec, ok := sections["with"]; ok {
		with = p.toWithSection(sec)
	}
	var given *ast.GivenSection
	if sec, ok := sections["given"]; ok {
		given = p.toGivenSection(sec)
	}
	var when *ast.WhenSection
	if sec, ok := sections["when"]; ok {
		when = p.toWhenSection(sec)
	}
	var suchThat *ast.SuchThatSection
	if sec, ok := sections["suchThat"]; ok {
		suchThat = p.toSuchThatSection(sec)
	}
	var means *ast.MeansSection
	if sec, ok := sections["means"]; ok {
		means = p.toMeansSection(sec)
	}
	var defines *ast.DefinesSection
	if sec, ok := sections["defines"]; ok {
		defines = p.toDefinesSection(sec)
	}
	var viewable *ast.ViewableSection
	if sec, ok := sections["Viewable"]; ok {
		viewable = p.toViewableSection(sec)
	}
	var provides *ast.ProvidesSection
	if sec, ok := sections["Provides"]; ok {
		provides = p.toProvidesSection(sec)
	}
	var using *ast.UsingSection
	if sec, ok := sections["Using"]; ok {
		using = p.toUsingSection(sec)
	}
	codified := *p.toCodifiedSection(sections["Codified"])
	var documented *ast.DocumentedSection
	if sec, ok := sections["Documented"]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var justified *ast.JustifiedSection
	if sec, ok := sections["Justified"]; ok {
		justified = p.toJustifiedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections["Justified"]; ok {
		references = p.toReferencesSection(sec)
	}
	var metadata *ast.MetadataSection
	if sec, ok := sections["Metadata"]; ok {
		metadata = p.toMetadataSection(sec)
	}
	return ast.DeclaresGroup{
		Id:         *id,
		Declares:   declares,
		With:       with,
		Given:      given,
		When:       when,
		SuchThat:   suchThat,
		Means:      means,
		Defines:    defines,
		Viewable:   viewable,
		Provides:   provides,
		Using:      using,
		Codified:   codified,
		Documented: documented,
		Justified:  justified,
		References: references,
		Metadata:   metadata,
	}, true
}

func (p *parser) toDeclaresSection(section phase4.Section) *ast.DeclaresSection {
	return &ast.DeclaresSection{
		Declares: p.exactlyOneTarget(section),
	}
}

func (p *parser) toWithSection(section phase4.Section) *ast.WithSection {
	return &ast.WithSection{
		With: p.oneOrMoreTargets(section),
	}
}

func (p *parser) toMeansSection(section phase4.Section) *ast.MeansSection {
	return &ast.MeansSection{
		Means: p.exactlyOneClause(section),
	}
}

func (p *parser) toDefinesSection(section phase4.Section) *ast.DefinesSection {
	return &ast.DefinesSection{
		Defines: p.oneOrMoreClauses(section),
	}
}

/////////////////////// states //////////////////////////////////////////

func (p *parser) toStatesGroup(group phase4.Group) (ast.StatesGroup, bool) {
	id := p.getId(group, true)
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"States",
		"given?",
		"when?",
		"suchThat?",
		"that",
		"Using?",
		"Codified",
		"Documented?",
		"Justified?",
		"References?",
		"Metadata?")
	if !ok || id == nil {
		return ast.StatesGroup{}, false
	}
	states := *p.toStatesSection(sections["States"])
	var given *ast.GivenSection
	if sec, ok := sections["given"]; ok {
		given = p.toGivenSection(sec)
	}
	var when *ast.WhenSection
	if sec, ok := sections["when"]; ok {
		when = p.toWhenSection(sec)
	}
	var suchThat *ast.SuchThatSection
	if sec, ok := sections["suchThat"]; ok {
		suchThat = p.toSuchThatSection(sec)
	}
	that := *p.toThatSection(sections["that"])
	var using *ast.UsingSection
	if sec, ok := sections["Using"]; ok {
		using = p.toUsingSection(sec)
	}
	codified := *p.toCodifiedSection(sections["Codified"])
	var documented *ast.DocumentedSection
	if sec, ok := sections["Documented"]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var justified *ast.JustifiedSection
	if sec, ok := sections["Justified"]; ok {
		justified = p.toJustifiedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections["References"]; ok {
		references = p.toReferencesSection(sec)
	}
	var metadata *ast.MetadataSection
	if sec, ok := sections["Metadata"]; ok {
		metadata = p.toMetadataSection(sec)
	}
	return ast.StatesGroup{
		Id:         *id,
		States:     states,
		Given:      given,
		When:       when,
		SuchThat:   suchThat,
		That:       that,
		Using:      using,
		Codified:   codified,
		Documented: documented,
		Justified:  justified,
		References: references,
		Metadata:   metadata,
	}, true
}

func (p *parser) toStatesSection(section phase4.Section) *ast.StatesSection {
	p.verifyNoArgs(section)
	return &ast.StatesSection{}
}

func (p *parser) toThatSection(section phase4.Section) *ast.ThatSection {
	return &ast.ThatSection{
		That: p.oneOrMoreClauses(section),
	}
}

/////////////////////// proof ///////////////////////////////////////////

// TODO: finish this

/////////////////////// axiom ///////////////////////////////////////////

func (p *parser) toAxiomGroup(group phase4.Group) (ast.AxiomGroup, bool) {
	id := p.getId(group, false)
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"Axiom",
		"given?",
		"where?",
		"suchThat?",
		"then",
		"iff?",
		"Using?",
		"Documented?",
		"References?",
		"Metadata?")
	if !ok {
		return ast.AxiomGroup{}, false
	}
	axiom := *p.toAxiomSection(sections["Axiom"])
	var given *ast.GivenSection
	if sec, ok := sections["given"]; ok {
		given = p.toGivenSection(sec)
	}
	var where *ast.WhereSection
	if sec, ok := sections["where"]; ok {
		where = p.toWhereSection(sec)
	}
	var suchThat *ast.SuchThatSection
	if sec, ok := sections["suchThat"]; ok {
		suchThat = p.toSuchThatSection(sec)
	}
	then := *p.toThenSection(sections["then"])
	var iff *ast.IffSection
	if sec, ok := sections["iff"]; ok {
		iff = p.toIffSection(sec)
	}
	var using *ast.UsingSection
	if sec, ok := sections["Using"]; ok {
		using = p.toUsingSection(sec)
	}
	var documented *ast.DocumentedSection
	if sec, ok := sections["Documented"]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections["References"]; ok {
		references = p.toReferencesSection(sec)
	}
	var metadata *ast.MetadataSection
	if sec, ok := sections["Metadata"]; ok {
		metadata = p.toMetadataSection(sec)
	}
	return ast.AxiomGroup{
		Id:         id,
		Axiom:      axiom,
		Given:      given,
		Where:      where,
		SuchThat:   suchThat,
		Then:       then,
		Iff:        iff,
		Using:      using,
		Documented: documented,
		References: references,
		Metadata:   metadata,
	}, true
}

func (p *parser) toAxiomSection(section phase4.Section) *ast.AxiomSection {
	return &ast.AxiomSection{
		Axiom: p.oneOrMoreTextItems(section),
	}
}

func (p *parser) toGivenSection(section phase4.Section) *ast.GivenSection {
	return &ast.GivenSection{
		Given: p.oneOrMoreTargets(section),
	}
}

/////////////////////// conjecture //////////////////////////////////////

func (p *parser) toConjectureGroup(group phase4.Group) (ast.ConjectureGroup, bool) {
	id := p.getId(group, false)
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"Conjecture",
		"given?",
		"where?",
		"suchThat?",
		"then",
		"iff?",
		"Using?",
		"Documented?",
		"References?",
		"Metadata?")
	if !ok {
		return ast.ConjectureGroup{}, false
	}
	conjecture := *p.toConjectureSection(sections["Conjecture"])
	var given *ast.GivenSection
	if sec, ok := sections["given"]; ok {
		given = p.toGivenSection(sec)
	}
	var where *ast.WhereSection
	if sec, ok := sections["where"]; ok {
		where = p.toWhereSection(sec)
	}
	var suchThat *ast.SuchThatSection
	if sec, ok := sections["suchThat"]; ok {
		suchThat = p.toSuchThatSection(sec)
	}
	then := *p.toThenSection(sections["then"])
	var iff *ast.IffSection
	if sec, ok := sections["iff"]; ok {
		iff = p.toIffSection(sec)
	}
	var using *ast.UsingSection
	if sec, ok := sections["Using"]; ok {
		using = p.toUsingSection(sec)
	}
	var documented *ast.DocumentedSection
	if sec, ok := sections["Documented"]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections["References"]; ok {
		references = p.toReferencesSection(sec)
	}
	var metadata *ast.MetadataSection
	if sec, ok := sections["Metadata"]; ok {
		metadata = p.toMetadataSection(sec)
	}
	return ast.ConjectureGroup{
		Id:         id,
		Conjecture: conjecture,
		Given:      given,
		Where:      where,
		SuchThat:   suchThat,
		Then:       then,
		Iff:        iff,
		Using:      using,
		Documented: documented,
		References: references,
		Metadata:   metadata,
	}, true
}

func (p *parser) toConjectureSection(section phase4.Section) *ast.ConjectureSection {
	return &ast.ConjectureSection{
		Conjecture: p.oneOrMoreTextItems(section),
	}
}

/////////////////////// theorem /////////////////////////////////////////

func (p *parser) toTheoremGroup(group phase4.Group) (ast.TheoremGroup, bool) {
	id := p.getId(group, false)
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"Theorem",
		"given?",
		"where?",
		"suchThat?",
		"then",
		"iff?",
		"Using?",
		"Proof",
		"Documented?",
		"References?",
		"Metadata?")
	if !ok {
		return ast.TheoremGroup{}, false
	}
	theorem := *p.toTheoremSection(sections["Theorem"])
	var given *ast.GivenSection
	if sec, ok := sections["given"]; ok {
		given = p.toGivenSection(sec)
	}
	var where *ast.WhereSection
	if sec, ok := sections["where"]; ok {
		where = p.toWhereSection(sec)
	}
	var suchThat *ast.SuchThatSection
	if sec, ok := sections["suchThat"]; ok {
		suchThat = p.toSuchThatSection(sec)
	}
	then := *p.toThenSection(sections["then"])
	var iff *ast.IffSection
	if sec, ok := sections["iff"]; ok {
		iff = p.toIffSection(sec)
	}
	var using *ast.UsingSection
	if sec, ok := sections["Using"]; ok {
		using = p.toUsingSection(sec)
	}
	var proof *ast.ProofSection
	if sec, ok := sections["Proof"]; ok {
		proof = p.toProofSection(sec)
	}
	var documented *ast.DocumentedSection
	if sec, ok := sections["Documented"]; ok {
		documented = p.toDocumentedSection(sec)
	}
	var references *ast.ReferencesSection
	if sec, ok := sections["References"]; ok {
		references = p.toReferencesSection(sec)
	}
	var metadata *ast.MetadataSection
	if sec, ok := sections["Metadata"]; ok {
		metadata = p.toMetadataSection(sec)
	}
	return ast.TheoremGroup{
		Id:         id,
		Theorem:    theorem,
		Given:      given,
		Where:      where,
		SuchThat:   suchThat,
		Then:       then,
		Iff:        iff,
		Using:      using,
		Proof:      proof,
		Documented: documented,
		References: references,
		Metadata:   metadata,
	}, true
}

func (p *parser) toTheoremSection(section phase4.Section) *ast.TheoremSection {
	return &ast.TheoremSection{
		Theorem: p.oneOrMoreTextItems(section),
	}
}

func (p *parser) toProofSection(section phase4.Section) *ast.ProofSection {
	return &ast.ProofSection{
		Proof: p.exactlyOneTextItem(section),
	}
}

////////////////////// topic ////////////////////////////////////////////

// TODO: finish this

////////////////////// note /////////////////////////////////////////////

// TODO: finish this

////////////////////// specify //////////////////////////////////////////

// TODO: finish this

///////////////////////////// document ///////////////////////////////////

func (p *parser) toDocument(root phase4.Root) (*ast.Document, bool) {
	return nil, false
}

///////////////////////////// id ////////////////////////////////////////

func (p *parser) toIdItem(text string) *ast.IdItem {
	if node, ok := formulation.ParseId(text, p.tracker); ok {
		return &ast.IdItem{
			RawText: text,
			Root:    node,
			Label:   nil,
		}
	} else {
		return nil
	}
}

func (p *parser) getId(group phase4.Group, required bool) *ast.IdItem {
	if required && group.Id == nil {
		p.tracker.Append(newError("Expected an id", group.MetaData.Start))
		return nil
	}
	return p.toIdItem(*group.Id)
}

////////////////////////// arguments ////////////////////////////////////

func (p *parser) toClause(arg phase4.Argument) ast.Clause {
	switch data := arg.Arg.(type) {
	case phase4.FormulationArgumentData:
		if node, ok := formulation.ParseExpression(data.Text, p.tracker); ok {
			return ast.Formulation[ast.NodeType]{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}
		} else {
			return ast.Formulation[ast.NodeType]{}
		}
	case phase4.Group:
		if grp, ok := p.toAllOfGroup(data); ok {
			return grp
		} else if grp, ok := p.toNotGroup(data); ok {
			return grp
		} else if grp, ok := p.toAnyOfGroup(data); ok {
			return grp
		} else if grp, ok := p.toOneOfGroup(data); ok {
			return grp
		} else if grp, ok := p.ToExistsGroup(data); ok {
			return grp
		}
	}

	p.tracker.Append(newError("Expected a clause", arg.MetaData.Start))
	return ast.Formulation[ast.NodeType]{}
}

func (p *parser) toSpec(arg phase4.Argument) ast.Spec {
	switch data := arg.Arg.(type) {
	case phase4.FormulationArgumentData:
		if node, ok := formulation.ParseExpression(data.Text, p.tracker); ok {
			return ast.Spec{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}
		} else {
			return ast.Spec{}
		}
	default:
		p.tracker.Append(newError("Expected a specification", arg.MetaData.Start))
		return ast.Spec{}
	}
}

func (p *parser) toTarget(arg phase4.Argument) ast.Target {
	switch data := arg.Arg.(type) {
	case phase4.ArgumentTextArgumentData:
		if node, ok := formulation.ParseForm(data.Text, p.tracker); ok {
			return ast.Target{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}
		} else {
			return ast.Target{}
		}
	default:
		p.tracker.Append(newError("Expected a target", arg.MetaData.Start))
		return ast.Target{}
	}
}

func (p *parser) toTextItem(arg phase4.Argument) ast.TextItem {
	switch data := arg.Arg.(type) {
	case phase4.TextArgumentData:
		return ast.TextItem{
			RawText: data.Text,
		}
	default:
		p.tracker.Append(newError("Expected a text item", arg.MetaData.Start))
		return ast.TextItem{}
	}
}

func (p *parser) toSignatureItem(arg phase4.Argument) ast.Formulation[ast.Signature] {
	switch data := arg.Arg.(type) {
	case phase4.FormulationArgumentData:
		if node, ok := formulation.ParseSignature(data.Text, p.tracker); ok {
			return ast.Formulation[ast.Signature]{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}
		} else {
			return ast.Formulation[ast.Signature]{}
		}
	}

	p.tracker.Append(newError("Expected a signature", arg.MetaData.Start))
	return ast.Formulation[ast.Signature]{}
}

//////////////////////// argument lists /////////////////////////////////

func (p *parser) toClauses(args []phase4.Argument) []ast.Clause {
	result := make([]ast.Clause, 0)
	for _, arg := range args {
		result = append(result, p.toClause(arg))
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

func (p *parser) toSignaturesItems(args []phase4.Argument) []ast.Formulation[ast.Signature] {
	result := make([]ast.Formulation[ast.Signature], 0)
	for _, arg := range args {
		result = append(result, p.toSignatureItem(arg))
	}
	return result
}

func (p *parser) toDocumentedTypes(args []phase4.Argument) []ast.DocumentedType {
	result := make([]ast.DocumentedType, 0)
	for _, arg := range args {
		if doc, ok := p.toDocumentedType(arg); ok {
			result = append(result, doc)
		} else {
			p.tracker.Append(newError("Expected a documented type", arg.MetaData.Start))
		}
	}
	return result
}

/////////////////////////////////////////////////////////////////////////

func (p *parser) verifyNoArgs(section phase4.Section) {
	if len(section.Args) > 0 {
		p.tracker.Append(newError("Expected no arguments", section.MetaData.Start))
	}
}

func (p *parser) oneOrMoreClauses(section phase4.Section) []ast.Clause {
	return oneOrMore(p.toClauses(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneClause(section phase4.Section) ast.Clause {
	var def ast.Clause = ast.Formulation[ast.NodeType]{}
	return exactlyOne(p.toClauses(section.Args), def, section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreSpecs(section phase4.Section) []ast.Spec {
	return oneOrMore(p.toSpecs(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneSpec(section phase4.Section) ast.Spec {
	var def ast.Spec = ast.Formulation[ast.NodeType]{}
	return exactlyOne(p.toSpecs(section.Args), def, section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreTargets(section phase4.Section) []ast.Target {
	return oneOrMore(p.toTargets(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreTextItems(section phase4.Section) []ast.TextItem {
	return oneOrMore(p.toTextItems(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneTarget(section phase4.Section) ast.Target {
	var def ast.Target = ast.Formulation[ast.NodeType]{}
	return exactlyOne(p.toTargets(section.Args), def, section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneSignatureItem(section phase4.Section) ast.Formulation[ast.Signature] {
	var def ast.Formulation[ast.Signature] = ast.Formulation[ast.Signature]{}
	return exactlyOne(p.toSignaturesItems(section.Args), def, section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreSignatureItems(section phase4.Section) []ast.Formulation[ast.Signature] {
	return oneOrMore(p.toSignaturesItems(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneTextItem(section phase4.Section) ast.TextItem {
	return exactlyOne(p.toTextItems(section.Args), ast.TextItem{}, section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreDocumentedTypes(section phase4.Section) []ast.DocumentedType {
	return oneOrMore(p.toDocumentedTypes(section.Args), section.MetaData.Start, p.tracker)
}

////////////////////////// support functions ////////////////////////////

func oneOrMore[T any](items []T, position ast.Position, tracker frontend.DiagnosticTracker) []T {
	if len(items) == 0 {
		tracker.Append(newError("Expected at least one item", position))
		return []T{}
	}
	return items
}

func exactlyOne[T any](items []T, defaultItem T, position ast.Position, tracker frontend.DiagnosticTracker) T {
	if len(items) != 1 {
		tracker.Append(newError("Expected at exactly one item", position))
	}
	if len(items) == 0 {
		return defaultItem
	}
	return items[0]
}

/*
func hasSections(group phase4.Group, names ...string) bool {
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
*/

func newError(message string, position ast.Position) frontend.Diagnostic {
	return frontend.Diagnostic{
		Type:     frontend.Error,
		Origin:   frontend.Phase5ParserOrigin,
		Message:  message,
		Position: position,
	}
}
