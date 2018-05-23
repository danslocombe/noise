// This is really ugly, because of a lack of metaprogramming
// It makes me sad
macro_rules! add_graph_fun {
    ($c:expr, $scope:expr, $name:expr, $func:ident, 0) => {
        let c_tmp = $c.clone();
        $scope.add_value_with_name($name, move |lisp_name| {
            let closure = move |_scope, args| {
                if (args.len() != 0) {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(0 as u32),
                        found: args.len() as u32,
                    }))
                }
                else {
                    $func(
                        c_tmp.borrow_mut(),
                        gc_tmp.borrow());
                    Ok(Value::Unit)
                }
            }
        });
    };
    ($c:expr, $gc:expr, $scope:expr, $name:expr, $func:ident, 1) => {
        let gc_tmp = $gc.clone();
        let c_tmp = $c.clone();
        $scope.add_value_with_name($name, move |lisp_name| {
            let closure = move |_scope, args| {
                if (args.len() != 1) {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(1 as u32),
                        found: args.len() as u32,
                    }))
                }
                else {
                    $func(
                        c_tmp.borrow_mut(),
                        gc_tmp.borrow(),
                        FromValueRef::from_value_ref(args[0])?);
                    Ok(Value::Unit)
                }
            }
        });
    };
    ($c:expr, $gc:expr, $scope:expr, $name:expr, $func:ident, 2) => {
        let gc_tmp = $gc.clone();
        let c_tmp = $c.clone();
        $scope.add_value_with_name($name, move |lisp_name| {
            Value::new_foreign_fn(lisp_name, move |_scope, args| {
                if (args.len() != 3) {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(5 as u32),
                        found: args.len() as u32,
                    }))
                }
                else {
                    $func(
                        c_tmp.borrow_mut(),
                        (*gc_tmp).borrow().clone(),
                        FromValueRef::from_value_ref(&args[0])?,
                        FromValueRef::from_value_ref(&args[1])?);
                    Ok(Value::Unit)
                }
            })
        });
    };
    ($c:expr, $gc:expr, $scope:expr, $name:expr, $func:ident, 3) => {
        let gc_tmp = $gc.clone();
        let c_tmp = $c.clone();
        $scope.add_value_with_name($name, move |lisp_name| {
            Value::new_foreign_fn(lisp_name, move |_scope, args| {
                if (args.len() != 3) {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(5 as u32),
                        found: args.len() as u32,
                    }))
                }
                else {
                    $func(
                        c_tmp.borrow_mut(),
                        (*gc_tmp).borrow().clone(),
                        FromValueRef::from_value_ref(&args[0])?,
                        FromValueRef::from_value_ref(&args[1])?,
                        FromValueRef::from_value_ref(&args[2])?);
                    Ok(Value::Unit)
                }
            })
        });
    };
    ($c:expr, $gc:expr, $scope:expr, $name:expr, $func:ident, 4) => {
        let gc_tmp = $gc.clone();
        let c_tmp = $c.clone();
        $scope.add_value_with_name($name, move |lisp_name| {
            Value::new_foreign_fn(lisp_name, move |_scope, args| {
                if (args.len() != 3) {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(5 as u32),
                        found: args.len() as u32,
                    }))
                }
                else {
                    $func(
                        c_tmp.borrow_mut(),
                        (*gc_tmp).borrow().clone(),
                        FromValueRef::from_value_ref(&args[0])?,
                        FromValueRef::from_value_ref(&args[1])?,
                        FromValueRef::from_value_ref(&args[2])?,
                        FromValueRef::from_value_ref(&args[3])?);
                    Ok(Value::Unit)
                }
            })
        });
    };
    ($c:expr, $gc:expr, $scope:expr, $name:expr, $func:ident, 5) => {
        let gc_tmp = $gc.clone();
        let c_tmp = $c.clone();
        $scope.add_value_with_name($name, move |lisp_name| {
            Value::new_foreign_fn(lisp_name, move |_scope, args| {
                if (args.len() != 5) {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(5 as u32),
                        found: args.len() as u32,
                    }))
                }
                else {
                    $func(
                        c_tmp.borrow_mut(),
                        (*gc_tmp).borrow().clone(),
                        FromValueRef::from_value_ref(&args[0])?,
                        FromValueRef::from_value_ref(&args[1])?,
                        FromValueRef::from_value_ref(&args[2])?,
                        FromValueRef::from_value_ref(&args[3])?,
                        FromValueRef::from_value_ref(&args[4])?);
                    Ok(Value::Unit)
                }
            })
        });
    };
}

macro_rules! add_context_mut {
    ($gc:expr, $scope:expr, $name:expr, $func:ident, 0) => {
        let gc_tmp = $gc.clone();
        $scope.add_value_with_name($name, move |lisp_name| {
            Value::new_foreign_fn(lisp_name, move |_scope, args| {
                if (args.len() != 0) {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(0 as u32),
                        found: args.len() as u32,
                    }))
                }
                else {
                    $func(
                        &mut gc_tmp.borrow_mut());
                    Ok(Value::Unit)
                }
            })
        });
    };
    ($gc:expr, $scope:expr, $name:expr, $func:ident, 1) => {
        let gc_tmp = $gc.clone();
        $scope.add_value_with_name($name, move |lisp_name| {
            Value::new_foreign_fn(lisp_name, move |_scope, args| {
                if (args.len() != 1) {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(1 as u32),
                        found: args.len() as u32,
                    }))
                }
                else {
                    $func(
                        &mut gc_tmp.borrow_mut(),
                        FromValueRef::from_value_ref(&args[0])?,
                        );
                    Ok(Value::Unit)
                }
            })
        });
    };
    ($gc:expr, $scope:expr, $name:expr, $func:ident, 2) => {
        let gc_tmp = $gc.clone();
        $scope.add_value_with_name($name, move |lisp_name| {
            Value::new_foreign_fn(lisp_name, move |_scope, args| {
                if (args.len() != 2) {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(2 as u32),
                        found: args.len() as u32,
                    }))
                }
                else {
                    $func(
                        &mut gc_tmp.borrow_mut(),
                        FromValueRef::from_value_ref(&args[0])?,
                        FromValueRef::from_value_ref(&args[1])?,
                        );
                    Ok(Value::Unit)
                }
            })
        });
    };
    ($gc:expr, $scope:expr, $name:expr, $func:ident, 3) => {
        let gc_tmp = $gc.clone();
        $scope.add_value_with_name($name, move |lisp_name| {
            Value::new_foreign_fn(lisp_name, move |_scope, args| {
                if (args.len() != 3) {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(3 as u32),
                        found: args.len() as u32,
                    }))
                }
                else {
                    $func(
                        &mut gc_tmp.borrow_mut(),
                        FromValueRef::from_value_ref(&args[0])?,
                        FromValueRef::from_value_ref(&args[1])?,
                        FromValueRef::from_value_ref(&args[2])?,
                        );
                    Ok(Value::Unit)
                }
            })
        });
    };
}
