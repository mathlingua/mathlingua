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
	"errors"
	"fmt"
	"mathlingua/internal/ast"
)

type ConstraintKind interface {
	ConstraintType()
}

func (*IsConstraint) ConstraintType()      {}
func (*SpecConstraint) ConstraintType()    {}
func (*ExtendsConstraint) ConstraintType() {}

type IsConstraint struct {
	Target       PatternKind
	SignatureExp ast.KindKind
	Scope        *ast.Scope
}

type ExtendsConstraint struct {
	Target       PatternKind
	SignatureExp ast.KindKind
	Scope        *ast.Scope
}

type SpecConstraint struct {
	Target PatternKind
	Name   string
	Exp    ast.ExpressionKind
	Scope  *ast.Scope
}

func ToIsConstraint(node ast.IsExpression) ([]IsConstraint, error) {
	result := make([]IsConstraint, 0)
	for _, lhsExp := range node.Lhs {
		for _, rhsExp := range node.Rhs {
			result = append(result, IsConstraint{
				Target:       ToPattern(lhsExp),
				SignatureExp: rhsExp,
				Scope:        node.CommonMetaData.Scope,
			})
		}
	}
	return result, nil
}

func ToExtendsConstraint(node ast.ExtendsExpression) ([]ExtendsConstraint, error) {
	result := make([]ExtendsConstraint, 0)
	for _, lhsExp := range node.Lhs {
		for _, rhsExp := range node.Rhs {
			result = append(result, ExtendsConstraint{
				Target:       ToPattern(lhsExp),
				SignatureExp: rhsExp,
				Scope:        node.CommonMetaData.Scope,
			})
		}
	}
	return result, nil
}

func ToSpecConstraint(node ast.InfixOperatorCallExpression) (SpecConstraint, error) {
	return SpecConstraint{
		Target: ToPattern(node.Lhs),
		Name:   node.Target.ToCode(noOp),
		Exp:    node.Rhs,
		Scope:  node.CommonMetaData.Scope,
	}, nil
}

func ToSpecConstraints(node ast.MultiplexedInfixOperatorCallExpression) ([]SpecConstraint, error) {
	result := make([]SpecConstraint, 0)
	for _, lhsExp := range node.Lhs {
		for _, rhsExp := range node.Rhs {
			result = append(result, SpecConstraint{
				Target: ToPattern(lhsExp),
				Name:   node.Target.ToCode(noOp),
				Exp:    rhsExp,
				Scope:  node.CommonMetaData.Scope,
			})
		}
	}
	return result, nil
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func noOp(node ast.MlgNodeKind) (string, bool) {
	return "", false
}

func toSingleStructuralForm(exp ast.ExpressionKind) (ast.StructuralFormKind, error) {
	nodes, err := toStructuralForms(exp)
	if err != nil {
		return nil, err
	}
	if len(nodes) != 1 {
		return nil, errors.New(fmt.Sprintf("Expected a single structural form but found %d",
			len(nodes)))
	}
	return nodes[0], nil
}

func toStructuralForms(exp ast.ExpressionKind) ([]ast.StructuralFormKind, error) {
	switch n := exp.(type) {
	case *ast.NameForm:
		return []ast.StructuralFormKind{n}, nil
	case *ast.FunctionCallExpression:
		target, err := toSingleStructuralForm(n.Target)
		if err != nil {
			return nil, err
		}
		name, err := toNameForm(target)
		if err != nil {
			return nil, err
		}
		args, err := toSingleStructuralFormSlice(n.Args)
		if err != nil {
			return nil, err
		}
		return []ast.StructuralFormKind{
			&ast.FunctionForm{
				Target: name,
				Params: args,
				// TODO: check vararg
			},
		}, nil
	case *ast.TupleExpression:
		params, err := toSingleStructuralFormSlice(n.Args)
		if err != nil {
			return nil, err
		}
		return []ast.StructuralFormKind{
			&ast.TupleForm{
				Params: params,
				// TODO: check vararg
			},
		}, nil
	case *ast.ConditionalSetExpression:
		target, err := toSingleStructuralForm(n.Target)
		if err != nil {
			return nil, err
		}
		return []ast.StructuralFormKind{
			&ast.ConditionalSetForm{
				Target: target,
				// TODO: check vararg
			},
		}, nil
	default:
		return nil, errors.New(fmt.Sprintf("Expected a structural form but found %s", exp.ToCode(noOp)))
	}
}

func toNameFormSlice(args []ast.StructuralFormKind) ([]ast.NameForm, error) {
	result := make([]ast.NameForm, 0)
	for _, arg := range args {
		name, err := toNameForm(arg)
		if err != nil {
			return nil, err
		}
		result = append(result, name)
	}
	return result, nil
}

func toNameForm(form ast.StructuralFormKind) (ast.NameForm, error) {
	name, ok := form.(*ast.NameForm)
	if ok {
		return *name, nil
	}
	return ast.NameForm{}, errors.New(fmt.Sprintf("Expected a name but found %s", form.ToCode(noOp)))
}

func toSingleStructuralFormSlice(args []ast.ExpressionKind) ([]ast.StructuralFormKind, error) {
	result := make([]ast.StructuralFormKind, 0)
	for _, arg := range args {
		form, err := toSingleStructuralForm(arg)
		if err != nil {
			return nil, err
		}
		result = append(result, form)
	}
	return result, nil
}
