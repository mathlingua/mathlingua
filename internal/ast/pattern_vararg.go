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

func (p *NameFormPattern) GetVarArgData() VarArgPatternData                 { return p.VarArg }
func (p *FunctionFormPattern) GetVarArgData() VarArgPatternData             { return p.VarArg }
func (p *ExpressionFormPattern) GetVarArgData() VarArgPatternData           { return p.VarArg }
func (p *TupleFormPattern) GetVarArgData() VarArgPatternData                { return p.VarArg }
func (p *ConditionalSetExpressionPattern) GetVarArgData() VarArgPatternData { return p.VarArg }
func (p *ConditionalSetFormPattern) GetVarArgData() VarArgPatternData       { return p.VarArg }

func (p *InfixOperatorFormPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *PrefixOperatorFormPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *PostfixOperatorFormPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *OrdinalPattern) GetVarArgData() VarArgPatternData { return VarArgPatternData{} }

func (p *StructuralColonEqualsPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *InfixCommandOperatorPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *InfixCommandPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *CommandPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *NamedGroupPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *ChainExpressionPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}

func (p *SpecAliasPattern) GetVarArgData() VarArgPatternData { return VarArgPatternData{} }
func (p *AliasPattern) GetVarArgData() VarArgPatternData     { return VarArgPatternData{} }

func (p *ConditionalSetIdFormPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}

func (p *FunctionLiteralFormPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
