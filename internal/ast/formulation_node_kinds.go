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
func (*FunctionForm) FormulationNodeKind()                           {}
func (*TupleForm) FormulationNodeKind()                              {}
func (*ConditionalSetForm) FormulationNodeKind()                     {}
func (*ConditionalSetIdForm) FormulationNodeKind()                   {}
func (*FunctionCallExpression) FormulationNodeKind()                 {}
func (*TupleExpression) FormulationNodeKind()                        {}
func (*ConditionalSetExpression) FormulationNodeKind()               {}
func (*CommandExpression) FormulationNodeKind()                      {}
func (*PrefixOperatorCallExpression) FormulationNodeKind()           {}
func (*PostfixOperatorCallExpression) FormulationNodeKind()          {}
func (*InfixOperatorCallExpression) FormulationNodeKind()            {}
func (*IsExpression) FormulationNodeKind()                           {}
func (*SatisfiesExpression) FormulationNodeKind()                    {}
func (*ExtendsExpression) FormulationNodeKind()                      {}
func (*AsExpression) FormulationNodeKind()                           {}
func (*OrdinalCallExpression) FormulationNodeKind()                  {}
func (*ChainExpression) FormulationNodeKind()                        {}
func (*Signature) FormulationNodeKind()                              {}
func (*TypeMetaKind) FormulationNodeKind()                           {}
func (*FormulationMetaKind) FormulationNodeKind()                    {}
func (*StructuralColonEqualsForm) FormulationNodeKind()              {}
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
func (*SelectFromBuiltinExpression) FormulationNodeKind()            {}
func (*MapToElseBuiltinExpression) FormulationNodeKind()             {}
func (*CommandType) FormulationNodeKind()                            {}
func (*InfixCommandType) FormulationNodeKind()                       {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type StructuralFormKind interface {
	FormulationNodeKind
	StructuralForm()
}

func (*NameForm) StructuralForm()                  {}
func (*FunctionForm) StructuralForm()              {}
func (*TupleForm) StructuralForm()                 {}
func (*ConditionalSetForm) StructuralForm()        {}
func (*ConditionalSetIdForm) StructuralForm()      {}
func (*InfixOperatorForm) StructuralForm()         {}
func (*PrefixOperatorForm) StructuralForm()        {}
func (*PostfixOperatorForm) StructuralForm()       {}
func (*FunctionLiteralForm) StructuralForm()       {}
func (*StructuralColonEqualsForm) StructuralForm() {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type LiteralFormKind interface {
	FormulationNodeKind
	LiteralFormKind()
}

func (*NameForm) LiteralFormKind()             {}
func (*FunctionForm) LiteralFormKind()         {}
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
func (*FunctionCallExpression) ExpressionKind()                 {}
func (*TupleExpression) ExpressionKind()                        {}
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
func (*SatisfiesExpression) ExpressionKind()                    {}
func (*ExtendsExpression) ExpressionKind()                      {}
func (*MultiplexedInfixOperatorCallExpression) ExpressionKind() {}
func (*ExpressionColonEqualsItem) ExpressionKind()              {}
func (*ExpressionColonArrowItem) ExpressionKind()               {}
func (*ExpressionColonDashArrowItem) ExpressionKind()           {}
func (*Signature) ExpressionKind()                              {}
func (*FunctionLiteralExpression) ExpressionKind()              {}
func (*SelectFromBuiltinExpression) ExpressionKind()            {}
func (*MapToElseBuiltinExpression) ExpressionKind()             {}
func (*CommandType) ExpressionKind()                            {}
func (*InfixCommandType) ExpressionKind()                       {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type TypeKind interface {
	ExpressionKind
	TypeKind()
}

func (*InfixCommandType) TypeKind()            {}
func (*CommandType) TypeKind()                 {}
func (*InfixOperatorCallExpression) TypeKind() {} // \:set \:in:/ \:set

////////////////////////////////////////////////////////////////////////////////////////////////////

type KindKind interface {
	FormulationNodeKind
	KindKind()
}

func (*NameForm) KindKind()                      {} // x could refer to a type
func (*CommandExpression) KindKind()             {} // \function:on{A}:to{B}
func (*PrefixOperatorCallExpression) KindKind()  {} // *A
func (*PostfixOperatorCallExpression) KindKind() {} // B!
func (*InfixOperatorCallExpression) KindKind()   {} // A \to/ B
func (*TypeMetaKind) KindKind()                  {} // \\type{\[set] & \[group]}
func (*FormulationMetaKind) KindKind()           {} // \\formulation{expression | statement}

////////////////////////////////////////////////////////////////////////////////////////////////////

type ColonEqualsKind interface {
	FormulationNodeKind
	ColonEqualsKind()
}

func (StructuralColonEqualsForm) ColonEqualsKind() {}
func (ExpressionColonEqualsItem) ColonEqualsKind() {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type OperatorKind interface {
	FormulationNodeKind
	OperatorKind()
}

func (*EnclosedNonCommandOperatorTarget) OperatorKind()    {}
func (*NonEnclosedNonCommandOperatorTarget) OperatorKind() {}
func (*InfixCommandExpression) OperatorKind()              {}
func (*InfixCommandType) OperatorKind()                    {}

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

type DirectionParamParamKind interface {
	FormulationNodeKind
	DirectionParamParamKind()
}

func (*NameForm) DirectionParamParamKind()              {}
func (*FunctionForm) DirectionParamParamKind()          {}
func (*OrdinalCallExpression) DirectionParamParamKind() {}
