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

type FormulationMetaData struct {
	Original FormulationNodeKind
}

func (n *NameForm) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *FunctionForm) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *TupleForm) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *ConditionalSetForm) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *ConditionalSetIdForm) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *FunctionCallExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *TupleExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *ConditionalSetExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *CommandExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *PrefixOperatorCallExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *PostfixOperatorCallExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *InfixOperatorCallExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *IsExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *ExtendsExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *AsExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *OrdinalCallExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *ChainExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *Signature) GetFormulationMetaData() *FormulationMetaData { return &n.FormulationMetaData }

func (n *TypeMetaKind) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *FormulationMetaKind) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *StructuralColonEqualsForm) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *ExpressionColonEqualsItem) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *ExpressionColonArrowItem) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *ExpressionColonDashArrowItem) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *EnclosedNonCommandOperatorTarget) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *NonEnclosedNonCommandOperatorTarget) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *InfixCommandExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *CommandId) GetFormulationMetaData() *FormulationMetaData { return &n.FormulationMetaData }

func (n *PrefixOperatorId) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *PostfixOperatorId) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *InfixOperatorId) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *InfixCommandOperatorId) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *PseudoTokenNode) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *PseudoExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *MultiplexedInfixOperatorCallExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *InfixOperatorForm) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *PrefixOperatorForm) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *PostfixOperatorForm) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *FunctionLiteralExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *FunctionLiteralForm) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *SelectFromBuiltinExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *MapToElseBuiltinExpression) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *CommandType) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}

func (n *InfixCommandType) GetFormulationMetaData() *FormulationMetaData {
	return &n.FormulationMetaData
}
