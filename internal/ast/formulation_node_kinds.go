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

type FormulationNodeKind interface {
	MlgNodeKind
	FormulationNodeKind()
	Start() Position
	GetFormulationMetaData() *FormulationMetaData
	ToCode(fn func(node MlgNodeKind) (string, bool)) string
}

func (*NameForm) FormulationNodeKind()                               {}
func (*SymbolForm) FormulationNodeKind()                             {}
func (*FunctionForm) FormulationNodeKind()                           {}
func (*ExpressionForm) FormulationNodeKind()                         {}
func (*TupleForm) FormulationNodeKind()                              {}
func (*ConditionalSetForm) FormulationNodeKind()                     {}
func (*ConditionalSetIdForm) FormulationNodeKind()                   {}
func (*FunctionCallExpression) FormulationNodeKind()                 {}
func (*TupleExpression) FormulationNodeKind()                        {}
func (*LabeledGrouping) FormulationNodeKind()                        {}
func (*ConditionalSetExpression) FormulationNodeKind()               {}
func (*CommandExpression) FormulationNodeKind()                      {}
func (*PrefixOperatorCallExpression) FormulationNodeKind()           {}
func (*PostfixOperatorCallExpression) FormulationNodeKind()          {}
func (*InfixOperatorCallExpression) FormulationNodeKind()            {}
func (*IsExpression) FormulationNodeKind()                           {}
func (*AsExpression) FormulationNodeKind()                           {}
func (*OrdinalCallExpression) FormulationNodeKind()                  {}
func (*ChainExpression) FormulationNodeKind()                        {}
func (*Signature) FormulationNodeKind()                              {}
func (*StructuralColonEqualsForm) FormulationNodeKind()              {}
func (*StructuralColonEqualsColonForm) FormulationNodeKind()         {}
func (*ExpressionColonEqualsItem) FormulationNodeKind()              {}
func (*ExpressionColonArrowItem) FormulationNodeKind()               {}
func (*ExpressionColonDashArrowItem) FormulationNodeKind()           {}
func (*EnclosedNonCommandOperatorTarget) FormulationNodeKind()       {}
func (*NonEnclosedNonCommandOperatorTarget) FormulationNodeKind()    {}
func (*InfixCommandExpression) FormulationNodeKind()                 {}
func (*CommandId) FormulationNodeKind()                              {}
func (*PrefixOperatorId) FormulationNodeKind()                       {}
func (*PostfixOperatorId) FormulationNodeKind()                      {}
func (*InfixOperatorId) FormulationNodeKind()                        {}
func (*InfixCommandOperatorId) FormulationNodeKind()                 {}
func (*PseudoTokenNode) FormulationNodeKind()                        {}
func (*PseudoExpression) FormulationNodeKind()                       {}
func (*MultiplexedInfixOperatorCallExpression) FormulationNodeKind() {}
func (*InfixOperatorForm) FormulationNodeKind()                      {}
func (*PrefixOperatorForm) FormulationNodeKind()                     {}
func (*PostfixOperatorForm) FormulationNodeKind()                    {}
func (*FunctionLiteralExpression) FormulationNodeKind()              {}
func (*FunctionLiteralForm) FormulationNodeKind()                    {}
func (*DefinitionBuiltinExpression) FormulationNodeKind()            {}
func (*MapToElseBuiltinExpression) FormulationNodeKind()             {}
func (*CommandTypeForm) FormulationNodeKind()                        {}
func (*InfixCommandTypeForm) FormulationNodeKind()                   {}
func (*AbstractBuiltinExpression) FormulationNodeKind()              {}
func (*SpecificationBuiltinExpression) FormulationNodeKind()         {}
func (*StatementBuiltinExpression) FormulationNodeKind()             {}
func (*ExpressionBuiltinExpression) FormulationNodeKind()            {}
func (*TypeBuiltinExpression) FormulationNodeKind()                  {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type StructuralFormKind interface {
	FormulationNodeKind
	StructuralForm()
}

func (*NameForm) StructuralForm()                       {}
func (*SymbolForm) StructuralForm()                     {}
func (*FunctionForm) StructuralForm()                   {}
func (*ExpressionForm) StructuralForm()                 {}
func (*TupleForm) StructuralForm()                      {}
func (*ConditionalSetForm) StructuralForm()             {}
func (*ConditionalSetIdForm) StructuralForm()           {}
func (*InfixOperatorForm) StructuralForm()              {}
func (*PrefixOperatorForm) StructuralForm()             {}
func (*PostfixOperatorForm) StructuralForm()            {}
func (*FunctionLiteralForm) StructuralForm()            {}
func (*StructuralColonEqualsForm) StructuralForm()      {}
func (*StructuralColonEqualsColonForm) StructuralForm() {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type LiteralFormKind interface {
	FormulationNodeKind
	LiteralFormKind()
}

func (*NameForm) LiteralFormKind()             {}
func (*FunctionForm) LiteralFormKind()         {}
func (*ExpressionForm) LiteralFormKind()       {}
func (*TupleForm) LiteralFormKind()            {}
func (*ConditionalSetIdForm) LiteralFormKind() {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type LiteralExpressionKind interface {
	FormulationNodeKind
	LiteralExpressionKind()
}

func (*FunctionCallExpression) LiteralExpressionKind()    {}
func (*TupleExpression) LiteralExpressionKind()           {}
func (*ConditionalSetExpression) LiteralExpressionKind()  {}
func (*ExpressionForm) LiteralExpressionKind()            {}
func (*FunctionLiteralExpression) LiteralExpressionKind() {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type LiteralKind interface {
	LiteralFormKind
	LiteralExpressionKind
	LiteralKind()
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type ExpressionKind interface {
	FormulationNodeKind
	ExpressionKind()
}

func (*NameForm) ExpressionKind()                               {}
func (*SymbolForm) ExpressionKind()                             {}
func (*FunctionCallExpression) ExpressionKind()                 {}
func (*ExpressionForm) ExpressionKind()                         {}
func (*TupleExpression) ExpressionKind()                        {}
func (*LabeledGrouping) ExpressionKind()                        {}
func (*ConditionalSetExpression) ExpressionKind()               {}
func (*CommandExpression) ExpressionKind()                      {}
func (*PrefixOperatorCallExpression) ExpressionKind()           {}
func (*PostfixOperatorCallExpression) ExpressionKind()          {}
func (*InfixOperatorCallExpression) ExpressionKind()            {}
func (*AsExpression) ExpressionKind()                           {}
func (*OrdinalCallExpression) ExpressionKind()                  {}
func (*ChainExpression) ExpressionKind()                        {}
func (*PseudoTokenNode) ExpressionKind()                        {}
func (*PseudoExpression) ExpressionKind()                       {}
func (*IsExpression) ExpressionKind()                           {}
func (*MultiplexedInfixOperatorCallExpression) ExpressionKind() {}
func (*ExpressionColonEqualsItem) ExpressionKind()              {}
func (*ExpressionColonArrowItem) ExpressionKind()               {}
func (*ExpressionColonDashArrowItem) ExpressionKind()           {}
func (*Signature) ExpressionKind()                              {}
func (*FunctionLiteralExpression) ExpressionKind()              {}
func (*DefinitionBuiltinExpression) ExpressionKind()            {}
func (*MapToElseBuiltinExpression) ExpressionKind()             {}
func (*CommandTypeForm) ExpressionKind()                        {}
func (*InfixCommandTypeForm) ExpressionKind()                   {}
func (*AbstractBuiltinExpression) ExpressionKind()              {}
func (*SpecificationBuiltinExpression) ExpressionKind()         {}
func (*StatementBuiltinExpression) ExpressionKind()             {}
func (*ExpressionBuiltinExpression) ExpressionKind()            {}
func (*TypeBuiltinExpression) ExpressionKind()                  {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type TypeFormKind interface {
	ExpressionKind
	TypeKind()
}

func (*InfixCommandTypeForm) TypeKind()        {}
func (*CommandTypeForm) TypeKind()             {}
func (*InfixOperatorCallExpression) TypeKind() {} // \:set \:in:/ \:set

////////////////////////////////////////////////////////////////////////////////////////////////////

type KindKind interface {
	FormulationNodeKind
	KindKind()
}

func (*NameForm) KindKind()                       {} // x could refer to a type
func (*CommandExpression) KindKind()              {} // \function:on{A}:to{B}
func (*PrefixOperatorCallExpression) KindKind()   {} // *A
func (*PostfixOperatorCallExpression) KindKind()  {} // B!
func (*InfixOperatorCallExpression) KindKind()    {} // A \to/ B
func (*AbstractBuiltinExpression) KindKind()      {} // \\abstract
func (*SpecificationBuiltinExpression) KindKind() {} // \\specification
func (*StatementBuiltinExpression) KindKind()     {} // \\statement
func (*ExpressionBuiltinExpression) KindKind()    {} // \\expression
func (*TypeBuiltinExpression) KindKind()          {} // \\type

////////////////////////////////////////////////////////////////////////////////////////////////////

type OperatorKind interface {
	FormulationNodeKind
	OperatorKind()
}

func (*EnclosedNonCommandOperatorTarget) OperatorKind()    {}
func (*NonEnclosedNonCommandOperatorTarget) OperatorKind() {}
func (*InfixCommandExpression) OperatorKind()              {}
func (*InfixCommandTypeForm) OperatorKind()                {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type IdKind interface {
	FormulationNodeKind
	IdKind()
}

func (*CommandId) IdKind()              {}
func (*PrefixOperatorId) IdKind()       {}
func (*PostfixOperatorId) IdKind()      {}
func (*InfixOperatorId) IdKind()        {}
func (*InfixCommandOperatorId) IdKind() {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type StructuralColonEqualsColonFormItemKind interface {
	FormulationNodeKind
	StructuralColonEqualsColonFormItemKind()
}

func (*FunctionForm) StructuralColonEqualsColonFormItemKind()   {}
func (*TupleForm) StructuralColonEqualsColonFormItemKind()      {}
func (*ExpressionForm) StructuralColonEqualsColonFormItemKind() {}
