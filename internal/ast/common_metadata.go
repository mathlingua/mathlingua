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

package ast

func (n *Root) GetCommonMetaData() *CommonMetaData               { return &n.CommonMetaData }
func (n *IdItem) GetCommonMetaData() *CommonMetaData             { return &n.CommonMetaData }
func (n *Target) GetCommonMetaData() *CommonMetaData             { return &n.CommonMetaData }
func (n *Spec) GetCommonMetaData() *CommonMetaData               { return &n.CommonMetaData }
func (n *Alias) GetCommonMetaData() *CommonMetaData              { return &n.CommonMetaData }
func (n *Formulation[T]) GetCommonMetaData() *CommonMetaData     { return &n.CommonMetaData }
func (n *TextItem) GetCommonMetaData() *CommonMetaData           { return &n.CommonMetaData }
func (n *LetGroup) GetCommonMetaData() *CommonMetaData           { return &n.CommonMetaData }
func (n *AllOfGroup) GetCommonMetaData() *CommonMetaData         { return &n.CommonMetaData }
func (n *EquivalentlyGroup) GetCommonMetaData() *CommonMetaData  { return &n.CommonMetaData }
func (n *NotGroup) GetCommonMetaData() *CommonMetaData           { return &n.CommonMetaData }
func (n *AnyOfGroup) GetCommonMetaData() *CommonMetaData         { return &n.CommonMetaData }
func (n *OneOfGroup) GetCommonMetaData() *CommonMetaData         { return &n.CommonMetaData }
func (n *ExistsGroup) GetCommonMetaData() *CommonMetaData        { return &n.CommonMetaData }
func (n *ExistsUniqueGroup) GetCommonMetaData() *CommonMetaData  { return &n.CommonMetaData }
func (n *ForAllGroup) GetCommonMetaData() *CommonMetaData        { return &n.CommonMetaData }
func (n *IfGroup) GetCommonMetaData() *CommonMetaData            { return &n.CommonMetaData }
func (n *IffGroup) GetCommonMetaData() *CommonMetaData           { return &n.CommonMetaData }
func (n *PiecewiseGroup) GetCommonMetaData() *CommonMetaData     { return &n.CommonMetaData }
func (n *WhenGroup) GetCommonMetaData() *CommonMetaData          { return &n.CommonMetaData }
func (n *SymbolWrittenGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }
func (n *ViewGroup) GetCommonMetaData() *CommonMetaData          { return &n.CommonMetaData }
func (n *EncodingGroup) GetCommonMetaData() *CommonMetaData      { return &n.CommonMetaData }
func (n *WrittenGroup) GetCommonMetaData() *CommonMetaData       { return &n.CommonMetaData }
func (n *CalledGroup) GetCommonMetaData() *CommonMetaData        { return &n.CommonMetaData }
func (n *ExpressedGroup) GetCommonMetaData() *CommonMetaData     { return &n.CommonMetaData }
func (n *OverviewGroup) GetCommonMetaData() *CommonMetaData      { return &n.CommonMetaData }
func (n *RelatedGroup) GetCommonMetaData() *CommonMetaData       { return &n.CommonMetaData }
func (n *LabelGroup) GetCommonMetaData() *CommonMetaData         { return &n.CommonMetaData }
func (n *ByGroup) GetCommonMetaData() *CommonMetaData            { return &n.CommonMetaData }
func (n *DescribesGroup) GetCommonMetaData() *CommonMetaData     { return &n.CommonMetaData }
func (n *DefinesGroup) GetCommonMetaData() *CommonMetaData       { return &n.CommonMetaData }
func (n *LowerDefineGroup) GetCommonMetaData() *CommonMetaData   { return &n.CommonMetaData }
func (n *StatesGroup) GetCommonMetaData() *CommonMetaData        { return &n.CommonMetaData }
func (n *ProofGroup) GetCommonMetaData() *CommonMetaData         { return &n.CommonMetaData }
func (n *AxiomGroup) GetCommonMetaData() *CommonMetaData         { return &n.CommonMetaData }
func (n *ConjectureGroup) GetCommonMetaData() *CommonMetaData    { return &n.CommonMetaData }
func (n *TheoremGroup) GetCommonMetaData() *CommonMetaData       { return &n.CommonMetaData }
func (n *CorollaryGroup) GetCommonMetaData() *CommonMetaData     { return &n.CommonMetaData }
func (n *LemmaGroup) GetCommonMetaData() *CommonMetaData         { return &n.CommonMetaData }
func (n *ZeroGroup) GetCommonMetaData() *CommonMetaData          { return &n.CommonMetaData }
func (n *PositiveIntGroup) GetCommonMetaData() *CommonMetaData   { return &n.CommonMetaData }
func (n *NegativeIntGroup) GetCommonMetaData() *CommonMetaData   { return &n.CommonMetaData }
func (n *PositiveFloatGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }
func (n *NegativeFloatGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }
func (n *SpecifyGroup) GetCommonMetaData() *CommonMetaData       { return &n.CommonMetaData }
func (n *PersonGroup) GetCommonMetaData() *CommonMetaData        { return &n.CommonMetaData }
func (n *NameGroup) GetCommonMetaData() *CommonMetaData          { return &n.CommonMetaData }
func (n *BiographyGroup) GetCommonMetaData() *CommonMetaData     { return &n.CommonMetaData }
func (n *ResourceGroup) GetCommonMetaData() *CommonMetaData      { return &n.CommonMetaData }
func (n *TitleGroup) GetCommonMetaData() *CommonMetaData         { return &n.CommonMetaData }
func (n *AuthorGroup) GetCommonMetaData() *CommonMetaData        { return &n.CommonMetaData }
func (n *OffsetGroup) GetCommonMetaData() *CommonMetaData        { return &n.CommonMetaData }
func (n *UrlGroup) GetCommonMetaData() *CommonMetaData           { return &n.CommonMetaData }
func (n *HomepageGroup) GetCommonMetaData() *CommonMetaData      { return &n.CommonMetaData }
func (n *TypeGroup) GetCommonMetaData() *CommonMetaData          { return &n.CommonMetaData }
func (n *EditorGroup) GetCommonMetaData() *CommonMetaData        { return &n.CommonMetaData }
func (n *EditionGroup) GetCommonMetaData() *CommonMetaData       { return &n.CommonMetaData }
func (n *InstitutionGroup) GetCommonMetaData() *CommonMetaData   { return &n.CommonMetaData }
func (n *JournalGroup) GetCommonMetaData() *CommonMetaData       { return &n.CommonMetaData }
func (n *PublisherGroup) GetCommonMetaData() *CommonMetaData     { return &n.CommonMetaData }
func (n *VolumeGroup) GetCommonMetaData() *CommonMetaData        { return &n.CommonMetaData }
func (n *MonthGroup) GetCommonMetaData() *CommonMetaData         { return &n.CommonMetaData }
func (n *YearGroup) GetCommonMetaData() *CommonMetaData          { return &n.CommonMetaData }
func (n *DescriptionGroup) GetCommonMetaData() *CommonMetaData   { return &n.CommonMetaData }
func (n *Document) GetCommonMetaData() *CommonMetaData           { return &n.CommonMetaData }
func (n *TextBlockItem) GetCommonMetaData() *CommonMetaData      { return &n.CommonMetaData }
func (n *CapturesGroup) GetCommonMetaData() *CommonMetaData      { return &n.CommonMetaData }

func (n *NameForm) GetCommonMetaData() *CommonMetaData                 { return &n.CommonMetaData }
func (n *FunctionForm) GetCommonMetaData() *CommonMetaData             { return &n.CommonMetaData }
func (n *TupleForm) GetCommonMetaData() *CommonMetaData                { return &n.CommonMetaData }
func (n *ConditionalSetForm) GetCommonMetaData() *CommonMetaData       { return &n.CommonMetaData }
func (n *ConditionalSetIdForm) GetCommonMetaData() *CommonMetaData     { return &n.CommonMetaData }
func (n *FunctionCallExpression) GetCommonMetaData() *CommonMetaData   { return &n.CommonMetaData }
func (n *TupleExpression) GetCommonMetaData() *CommonMetaData          { return &n.CommonMetaData }
func (n *ConditionalSetExpression) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }
func (n *CommandExpression) GetCommonMetaData() *CommonMetaData        { return &n.CommonMetaData }

func (n *PrefixOperatorCallExpression) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *PostfixOperatorCallExpression) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *InfixOperatorCallExpression) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *IsExpression) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *ExtendsExpression) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *AsExpression) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *OrdinalCallExpression) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *ChainExpression) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *Signature) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *MetaKinds) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *StructuralColonEqualsForm) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *ExpressionColonEqualsItem) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *ExpressionColonArrowItem) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *ExpressionColonDashArrowItem) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *EnclosedNonCommandOperatorTarget) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *NonEnclosedNonCommandOperatorTarget) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *InfixCommandExpression) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *CommandId) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *PrefixOperatorId) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *PostfixOperatorId) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *InfixOperatorId) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *InfixCommandOperatorId) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *PseudoTokenNode) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *PseudoExpression) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *MultiplexedInfixOperatorCallExpression) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *ProofWithoutLossOfGeneralityGroup) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *InfixOperatorForm) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *PrefixOperatorForm) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *PostfixOperatorForm) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *NamedArg) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *NamedParam) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *InfixCommandId) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *FunctionLiteralExpression) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *CurlyParam) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *CurlyArg) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *DirectionalParam) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *FunctionLiteralForm) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofThenByGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofThusByGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofThereforeByGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofHenceByGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofNoticeByGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofNextByGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofThenBecauseGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofThusBecauseGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofThereforeBecauseGroup) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *ProofHenceBecauseGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofNoticeBecauseGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofNextBecauseGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofByThenGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofBecauseThenGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofIndependentlyGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofChainGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofSupposeGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofBlockGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofCasewiseGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofQedGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofContradictionGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofDoneGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofForContradictionGroup) GetCommonMetaData() *CommonMetaData {
	return &n.CommonMetaData
}

func (n *ProofForInductionGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofClaimGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofEquivalentlyGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofAllOfGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofNotGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofAnyOfGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofOneOfGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofExistsGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofExistsUniqueGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofForAllGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofLetGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofIfGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }

func (n *ProofIffGroup) GetCommonMetaData() *CommonMetaData { return &n.CommonMetaData }
