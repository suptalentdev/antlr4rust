grammar Labels;
         z : s[0] ;
		 s[isize v] : q=e {println!("{}",$e.v)};
		 e returns [isize v]
		   : a=e op='*' b=e {$v = $a.v * $b.v;}  # mult
		   | a=e '+' b=e {$v = $a.v + $b.v;}     # add
		   | INT         {$v = $INT.int;}        # anInt
		   | '(' x=e ')' {$v = $x.v;}            # parens
		   | x=e '++'    {$v = $x.v+1;}          # inc
		   | e '--'                              # dec
		   | ID          {$v = 3;}               # anID
		   ;
		 ID : 'a'..'z'+ ;
		 INT : '0'..'9'+ ;
		 WS : (' '|'\n') -> skip ;