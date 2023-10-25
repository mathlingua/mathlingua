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

type FormulationMetaData struct {
	Original FormulationNodeKind
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
