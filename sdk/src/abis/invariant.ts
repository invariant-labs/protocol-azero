export const abi = `
{
  "source": {
    "hash": "0x4f19a0152491660087fd1d38d163ab436967e96ac14a07220133afc86e2bd9ec",
    "language": "ink! 5.0.0",
    "compiler": "rustc 1.77.0",
    "build_info": {
      "build_mode": "Release",
      "cargo_contract_version": "4.1.1",
      "rust_toolchain": "stable-x86_64-unknown-linux-gnu",
      "wasm_opt_settings": {
        "keep_debug_symbols": false,
        "optimization_passes": "Z"
      }
    }
  },
  "contract": {
    "name": "invariant",
    "version": "0.1.0",
    "authors": [
      "Invariant Labs"
    ]
  },
  "image": null,
  "spec": {
    "constructors": [
      {
        "args": [
          {
            "label": "protocol_fee",
            "type": {
              "displayName": [
                "Percentage"
              ],
              "type": 18
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "new",
        "payable": false,
        "returnType": {
          "displayName": [
            "ink_primitives",
            "ConstructorResult"
          ],
          "type": 55
        },
        "selector": "0x9bae9d5e"
      }
    ],
    "docs": [],
    "environment": {
      "accountId": {
        "displayName": [
          "AccountId"
        ],
        "type": 2
      },
      "balance": {
        "displayName": [
          "Balance"
        ],
        "type": 11
      },
      "blockNumber": {
        "displayName": [
          "BlockNumber"
        ],
        "type": 0
      },
      "chainExtension": {
        "displayName": [
          "ChainExtension"
        ],
        "type": 112
      },
      "hash": {
        "displayName": [
          "Hash"
        ],
        "type": 110
      },
      "maxEventTopics": 4,
      "staticBufferSize": 16384,
      "timestamp": {
        "displayName": [
          "Timestamp"
        ],
        "type": 9
      }
    },
    "events": [
      {
        "args": [
          {
            "docs": [],
            "indexed": true,
            "label": "timestamp",
            "type": {
              "displayName": [
                "u64"
              ],
              "type": 9
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "address",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "pool",
            "type": {
              "displayName": [
                "PoolKey"
              ],
              "type": 16
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "liquidity",
            "type": {
              "displayName": [
                "Liquidity"
              ],
              "type": 19
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "lower_tick",
            "type": {
              "displayName": [
                "i32"
              ],
              "type": 12
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "upper_tick",
            "type": {
              "displayName": [
                "i32"
              ],
              "type": 12
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "current_sqrt_price",
            "type": {
              "displayName": [
                "SqrtPrice"
              ],
              "type": 27
            }
          }
        ],
        "docs": [],
        "label": "CreatePositionEvent",
        "module_path": "invariant::contracts::events",
        "signature_topic": "0x50a25822f8984babdbc09246e1d170630167a27235d98a5ff8ac7516a5cdab15"
      },
      {
        "args": [
          {
            "docs": [],
            "indexed": true,
            "label": "timestamp",
            "type": {
              "displayName": [
                "u64"
              ],
              "type": 9
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "address",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "pool",
            "type": {
              "displayName": [
                "PoolKey"
              ],
              "type": 16
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "indexes",
            "type": {
              "displayName": [
                "Vec"
              ],
              "type": 102
            }
          }
        ],
        "docs": [],
        "label": "CrossTickEvent",
        "module_path": "invariant::contracts::events",
        "signature_topic": "0xcccff012aed0ec795ebacacf32bce7106d512938af21db8ed9c2db1d2673378d"
      },
      {
        "args": [
          {
            "docs": [],
            "indexed": true,
            "label": "timestamp",
            "type": {
              "displayName": [
                "u64"
              ],
              "type": 9
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "address",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "pool",
            "type": {
              "displayName": [
                "PoolKey"
              ],
              "type": 16
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "liquidity",
            "type": {
              "displayName": [
                "Liquidity"
              ],
              "type": 19
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "lower_tick",
            "type": {
              "displayName": [
                "i32"
              ],
              "type": 12
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "upper_tick",
            "type": {
              "displayName": [
                "i32"
              ],
              "type": 12
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "current_sqrt_price",
            "type": {
              "displayName": [
                "SqrtPrice"
              ],
              "type": 27
            }
          }
        ],
        "docs": [],
        "label": "RemovePositionEvent",
        "module_path": "invariant::contracts::events",
        "signature_topic": "0x9f0ecfca7dad4ac575484802040e0a5b1ce0a4c53a3e2cc6bb41ccb6e9a5db12"
      },
      {
        "args": [
          {
            "docs": [],
            "indexed": true,
            "label": "timestamp",
            "type": {
              "displayName": [
                "u64"
              ],
              "type": 9
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "address",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "pool",
            "type": {
              "displayName": [
                "PoolKey"
              ],
              "type": 16
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "amount_in",
            "type": {
              "displayName": [
                "TokenAmount"
              ],
              "type": 23
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "amount_out",
            "type": {
              "displayName": [
                "TokenAmount"
              ],
              "type": 23
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "fee",
            "type": {
              "displayName": [
                "TokenAmount"
              ],
              "type": 23
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "start_sqrt_price",
            "type": {
              "displayName": [
                "SqrtPrice"
              ],
              "type": 27
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "target_sqrt_price",
            "type": {
              "displayName": [
                "SqrtPrice"
              ],
              "type": 27
            }
          },
          {
            "docs": [],
            "indexed": false,
            "label": "x_to_y",
            "type": {
              "displayName": [
                "bool"
              ],
              "type": 34
            }
          }
        ],
        "docs": [],
        "label": "SwapEvent",
        "module_path": "invariant::contracts::events",
        "signature_topic": "0xa2fa68a09109e5201d1e015005173040754b008fc2dc7203c0da57236f0ba81e"
      },
      {
        "args": [
          {
            "docs": [
              "Account providing allowance."
            ],
            "indexed": true,
            "label": "owner",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          },
          {
            "docs": [
              "Allowance beneficiary."
            ],
            "indexed": true,
            "label": "spender",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          },
          {
            "docs": [
              "New allowance amount."
            ],
            "indexed": false,
            "label": "amount",
            "type": {
              "displayName": [
                "u128"
              ],
              "type": 11
            }
          }
        ],
        "docs": [
          "Event emitted when allowance by 'owner' to 'spender' changes."
        ],
        "label": "Approval",
        "module_path": "token::events",
        "signature_topic": "0x25cdb6c93882e925abbfc9a8b7c85884b73c038c03a2492f238a5e5ba3fbff8c"
      },
      {
        "args": [
          {
            "docs": [
              "Transfer sender. 'None' in case of minting new tokens."
            ],
            "indexed": true,
            "label": "from",
            "type": {
              "displayName": [
                "Option"
              ],
              "type": 111
            }
          },
          {
            "docs": [
              "Transfer recipient. 'None' in case of burning tokens."
            ],
            "indexed": true,
            "label": "to",
            "type": {
              "displayName": [
                "Option"
              ],
              "type": 111
            }
          },
          {
            "docs": [
              "Amount of tokens transferred (or minted/burned)."
            ],
            "indexed": false,
            "label": "value",
            "type": {
              "displayName": [
                "u128"
              ],
              "type": 11
            }
          }
        ],
        "docs": [
          "Event emitted when transfer of tokens occurs."
        ],
        "label": "Transfer",
        "module_path": "token::events",
        "signature_topic": "0x990df076cb1e9527aa102cd100c1481efe393eeabb5825f9af1f5e58221864de"
      }
    ],
    "lang_error": {
      "displayName": [
        "ink",
        "LangError"
      ],
      "type": 56
    },
    "messages": [
      {
        "args": [],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::get_protocol_fee",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 57
        },
        "selector": "0xe285b69a"
      },
      {
        "args": [
          {
            "label": "pool_key",
            "type": {
              "displayName": [
                "PoolKey"
              ],
              "type": 16
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::withdraw_protocol_fee",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 58
        },
        "selector": "0x5a059796"
      },
      {
        "args": [
          {
            "label": "protocol_fee",
            "type": {
              "displayName": [
                "Percentage"
              ],
              "type": 18
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::change_protocol_fee",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 58
        },
        "selector": "0xc8bd0f58"
      },
      {
        "args": [
          {
            "label": "pool_key",
            "type": {
              "displayName": [
                "PoolKey"
              ],
              "type": 16
            }
          },
          {
            "label": "fee_receiver",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::change_fee_receiver",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 58
        },
        "selector": "0x0ebd3ec6"
      },
      {
        "args": [
          {
            "label": "pool_key",
            "type": {
              "displayName": [
                "PoolKey"
              ],
              "type": 16
            }
          },
          {
            "label": "lower_tick",
            "type": {
              "displayName": [
                "i32"
              ],
              "type": 12
            }
          },
          {
            "label": "upper_tick",
            "type": {
              "displayName": [
                "i32"
              ],
              "type": 12
            }
          },
          {
            "label": "liquidity_delta",
            "type": {
              "displayName": [
                "Liquidity"
              ],
              "type": 19
            }
          },
          {
            "label": "slippage_limit_lower",
            "type": {
              "displayName": [
                "SqrtPrice"
              ],
              "type": 27
            }
          },
          {
            "label": "slippage_limit_upper",
            "type": {
              "displayName": [
                "SqrtPrice"
              ],
              "type": 27
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::create_position",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 61
        },
        "selector": "0x0a1ca76b"
      },
      {
        "args": [
          {
            "label": "pool_key",
            "type": {
              "displayName": [
                "PoolKey"
              ],
              "type": 16
            }
          },
          {
            "label": "x_to_y",
            "type": {
              "displayName": [
                "bool"
              ],
              "type": 34
            }
          },
          {
            "label": "amount",
            "type": {
              "displayName": [
                "TokenAmount"
              ],
              "type": 23
            }
          },
          {
            "label": "by_amount_in",
            "type": {
              "displayName": [
                "bool"
              ],
              "type": 34
            }
          },
          {
            "label": "sqrt_price_limit",
            "type": {
              "displayName": [
                "SqrtPrice"
              ],
              "type": 27
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::swap",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 63
        },
        "selector": "0x1c590004"
      },
      {
        "args": [
          {
            "label": "amount_in",
            "type": {
              "displayName": [
                "TokenAmount"
              ],
              "type": 23
            }
          },
          {
            "label": "expected_amount_out",
            "type": {
              "displayName": [
                "TokenAmount"
              ],
              "type": 23
            }
          },
          {
            "label": "slippage",
            "type": {
              "displayName": [
                "Percentage"
              ],
              "type": 18
            }
          },
          {
            "label": "swaps",
            "type": {
              "displayName": [
                "Vec"
              ],
              "type": 67
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::swap_route",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 58
        },
        "selector": "0x5ff8d655"
      },
      {
        "args": [
          {
            "label": "pool_key",
            "type": {
              "displayName": [
                "PoolKey"
              ],
              "type": 16
            }
          },
          {
            "label": "x_to_y",
            "type": {
              "displayName": [
                "bool"
              ],
              "type": 34
            }
          },
          {
            "label": "amount",
            "type": {
              "displayName": [
                "TokenAmount"
              ],
              "type": 23
            }
          },
          {
            "label": "by_amount_in",
            "type": {
              "displayName": [
                "bool"
              ],
              "type": 34
            }
          },
          {
            "label": "sqrt_price_limit",
            "type": {
              "displayName": [
                "SqrtPrice"
              ],
              "type": 27
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::quote",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 69
        },
        "selector": "0xa2bd3fc7"
      },
      {
        "args": [
          {
            "label": "amount_in",
            "type": {
              "displayName": [
                "TokenAmount"
              ],
              "type": 23
            }
          },
          {
            "label": "swaps",
            "type": {
              "displayName": [
                "Vec"
              ],
              "type": 67
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::quote_route",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 72
        },
        "selector": "0x879b5256"
      },
      {
        "args": [
          {
            "label": "index",
            "type": {
              "displayName": [
                "u32"
              ],
              "type": 0
            }
          },
          {
            "label": "receiver",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::transfer_position",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 58
        },
        "selector": "0xe5af145a"
      },
      {
        "args": [
          {
            "label": "owner_id",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          },
          {
            "label": "index",
            "type": {
              "displayName": [
                "u32"
              ],
              "type": 0
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::get_position",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 61
        },
        "selector": "0xccb84930"
      },
      {
        "args": [
          {
            "label": "owner_id",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          },
          {
            "label": "size",
            "type": {
              "displayName": [
                "u32"
              ],
              "type": 0
            }
          },
          {
            "label": "offset",
            "type": {
              "displayName": [
                "u32"
              ],
              "type": 0
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::get_positions",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 74
        },
        "selector": "0xb2155f6b"
      },
      {
        "args": [
          {
            "label": "index",
            "type": {
              "displayName": [
                "u32"
              ],
              "type": 0
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::claim_fee",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 79
        },
        "selector": "0x4eb580e1"
      },
      {
        "args": [
          {
            "label": "index",
            "type": {
              "displayName": [
                "u32"
              ],
              "type": 0
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::remove_position",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 79
        },
        "selector": "0xfe63d239"
      },
      {
        "args": [
          {
            "label": "fee_tier",
            "type": {
              "displayName": [
                "FeeTier"
              ],
              "type": 17
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::add_fee_tier",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 58
        },
        "selector": "0x009b6a3d"
      },
      {
        "args": [
          {
            "label": "fee_tier",
            "type": {
              "displayName": [
                "FeeTier"
              ],
              "type": 17
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::remove_fee_tier",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 58
        },
        "selector": "0x3b497d6e"
      },
      {
        "args": [
          {
            "label": "fee_tier",
            "type": {
              "displayName": [
                "FeeTier"
              ],
              "type": 17
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::fee_tier_exist",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 82
        },
        "selector": "0x4e9e07ce"
      },
      {
        "args": [
          {
            "label": "token_0",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          },
          {
            "label": "token_1",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          },
          {
            "label": "fee_tier",
            "type": {
              "displayName": [
                "FeeTier"
              ],
              "type": 17
            }
          },
          {
            "label": "init_sqrt_price",
            "type": {
              "displayName": [
                "SqrtPrice"
              ],
              "type": 27
            }
          },
          {
            "label": "init_tick",
            "type": {
              "displayName": [
                "i32"
              ],
              "type": 12
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::create_pool",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 58
        },
        "selector": "0x98b595e9"
      },
      {
        "args": [
          {
            "label": "token_0",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          },
          {
            "label": "token_1",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          },
          {
            "label": "fee_tier",
            "type": {
              "displayName": [
                "FeeTier"
              ],
              "type": 17
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::get_pool",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 83
        },
        "selector": "0xf91e4a49"
      },
      {
        "args": [
          {
            "label": "token0",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          },
          {
            "label": "token1",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::get_all_pools_for_pair",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 85
        },
        "selector": "0xb62aa10d"
      },
      {
        "args": [
          {
            "label": "key",
            "type": {
              "displayName": [
                "PoolKey"
              ],
              "type": 16
            }
          },
          {
            "label": "index",
            "type": {
              "displayName": [
                "i32"
              ],
              "type": 12
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::get_tick",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 89
        },
        "selector": "0xeebd620b"
      },
      {
        "args": [
          {
            "label": "key",
            "type": {
              "displayName": [
                "PoolKey"
              ],
              "type": 16
            }
          },
          {
            "label": "index",
            "type": {
              "displayName": [
                "i32"
              ],
              "type": 12
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::is_tick_initialized",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 82
        },
        "selector": "0xdbae15e5"
      },
      {
        "args": [
          {
            "label": "size",
            "type": {
              "displayName": [
                "u16"
              ],
              "type": 10
            }
          },
          {
            "label": "offset",
            "type": {
              "displayName": [
                "u16"
              ],
              "type": 10
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::get_pool_keys",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 91
        },
        "selector": "0x57d47dcb"
      },
      {
        "args": [],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::get_fee_tiers",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 95
        },
        "selector": "0xd05b6003"
      },
      {
        "args": [
          {
            "label": "owner",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          },
          {
            "label": "index",
            "type": {
              "displayName": [
                "u32"
              ],
              "type": 0
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::get_position_with_associates",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 96
        },
        "selector": "0x96ccf001"
      },
      {
        "args": [
          {
            "label": "pool_key",
            "type": {
              "displayName": [
                "PoolKey"
              ],
              "type": 16
            }
          },
          {
            "label": "lower_tick_index",
            "type": {
              "displayName": [
                "i32"
              ],
              "type": 12
            }
          },
          {
            "label": "upper_tick_index",
            "type": {
              "displayName": [
                "i32"
              ],
              "type": 12
            }
          },
          {
            "label": "x_to_y",
            "type": {
              "displayName": [
                "bool"
              ],
              "type": 34
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::get_tickmap",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 99
        },
        "selector": "0x3b83b256"
      },
      {
        "args": [
          {
            "label": "pool_key",
            "type": {
              "displayName": [
                "PoolKey"
              ],
              "type": 16
            }
          },
          {
            "label": "tickmap",
            "type": {
              "displayName": [
                "Vec"
              ],
              "type": 102
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::get_liquidity_ticks",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 103
        },
        "selector": "0x8ef8d345"
      },
      {
        "args": [
          {
            "label": "owner",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::get_user_position_amount",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 107
        },
        "selector": "0xd842f034"
      },
      {
        "args": [
          {
            "label": "pool_key",
            "type": {
              "displayName": [
                "PoolKey"
              ],
              "type": 16
            }
          },
          {
            "label": "lower_tick",
            "type": {
              "displayName": [
                "i32"
              ],
              "type": 12
            }
          },
          {
            "label": "upper_tick",
            "type": {
              "displayName": [
                "i32"
              ],
              "type": 12
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::get_liquidity_ticks_amount",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 108
        },
        "selector": "0xf360dea7"
      },
      {
        "args": [
          {
            "label": "address",
            "type": {
              "displayName": [
                "AccountId"
              ],
              "type": 2
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::withdraw_all_wazero",
        "mutates": false,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 58
        },
        "selector": "0x06099ca3"
      },
      {
        "args": [
          {
            "label": "code_hash",
            "type": {
              "displayName": [
                "Hash"
              ],
              "type": 110
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::set_code",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 58
        },
        "selector": "0xe33a2343"
      },
      {
        "args": [
          {
            "label": "index",
            "type": {
              "displayName": [
                "u32"
              ],
              "type": 0
            }
          }
        ],
        "default": false,
        "docs": [],
        "label": "InvariantTrait::update_position_seconds_per_liquidity",
        "mutates": true,
        "payable": false,
        "returnType": {
          "displayName": [
            "ink",
            "MessageResult"
          ],
          "type": 58
        },
        "selector": "0x292f3055"
      }
    ]
  },
  "storage": {
    "root": {
      "layout": {
        "struct": {
          "fields": [
            {
              "layout": {
                "struct": {
                  "fields": [
                    {
                      "layout": {
                        "root": {
                          "layout": {
                            "leaf": {
                              "key": "0xf93ac913",
                              "ty": 0
                            }
                          },
                          "root_key": "0xf93ac913",
                          "ty": 1
                        }
                      },
                      "name": "positions_length"
                    },
                    {
                      "layout": {
                        "root": {
                          "layout": {
                            "struct": {
                              "fields": [
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xaa6cd0bf",
                                              "ty": 2
                                            }
                                          },
                                          "name": "token_x"
                                        },
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xaa6cd0bf",
                                              "ty": 2
                                            }
                                          },
                                          "name": "token_y"
                                        },
                                        {
                                          "layout": {
                                            "struct": {
                                              "fields": [
                                                {
                                                  "layout": {
                                                    "struct": {
                                                      "fields": [
                                                        {
                                                          "layout": {
                                                            "leaf": {
                                                              "key": "0xaa6cd0bf",
                                                              "ty": 9
                                                            }
                                                          },
                                                          "name": "0"
                                                        }
                                                      ],
                                                      "name": "Percentage"
                                                    }
                                                  },
                                                  "name": "fee"
                                                },
                                                {
                                                  "layout": {
                                                    "leaf": {
                                                      "key": "0xaa6cd0bf",
                                                      "ty": 10
                                                    }
                                                  },
                                                  "name": "tick_spacing"
                                                }
                                              ],
                                              "name": "FeeTier"
                                            }
                                          },
                                          "name": "fee_tier"
                                        }
                                      ],
                                      "name": "PoolKey"
                                    }
                                  },
                                  "name": "pool_key"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xaa6cd0bf",
                                              "ty": 11
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "Liquidity"
                                    }
                                  },
                                  "name": "liquidity"
                                },
                                {
                                  "layout": {
                                    "leaf": {
                                      "key": "0xaa6cd0bf",
                                      "ty": 12
                                    }
                                  },
                                  "name": "lower_tick_index"
                                },
                                {
                                  "layout": {
                                    "leaf": {
                                      "key": "0xaa6cd0bf",
                                      "ty": 12
                                    }
                                  },
                                  "name": "upper_tick_index"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "struct": {
                                              "fields": [
                                                {
                                                  "layout": {
                                                    "array": {
                                                      "layout": {
                                                        "leaf": {
                                                          "key": "0xaa6cd0bf",
                                                          "ty": 9
                                                        }
                                                      },
                                                      "len": 4,
                                                      "offset": "0xaa6cd0bf"
                                                    }
                                                  },
                                                  "name": "0"
                                                }
                                              ],
                                              "name": "U256"
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "FeeGrowth"
                                    }
                                  },
                                  "name": "fee_growth_inside_x"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "struct": {
                                              "fields": [
                                                {
                                                  "layout": {
                                                    "array": {
                                                      "layout": {
                                                        "leaf": {
                                                          "key": "0xaa6cd0bf",
                                                          "ty": 9
                                                        }
                                                      },
                                                      "len": 4,
                                                      "offset": "0xaa6cd0bf"
                                                    }
                                                  },
                                                  "name": "0"
                                                }
                                              ],
                                              "name": "U256"
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "FeeGrowth"
                                    }
                                  },
                                  "name": "fee_growth_inside_y"
                                },
                                {
                                  "layout": {
                                    "leaf": {
                                      "key": "0xaa6cd0bf",
                                      "ty": 9
                                    }
                                  },
                                  "name": "last_block_number"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xaa6cd0bf",
                                              "ty": 11
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "TokenAmount"
                                    }
                                  },
                                  "name": "tokens_owed_x"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xaa6cd0bf",
                                              "ty": 11
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "TokenAmount"
                                    }
                                  },
                                  "name": "tokens_owed_y"
                                },
                                {
                                  "layout": {
                                    "leaf": {
                                      "key": "0xaa6cd0bf",
                                      "ty": 9
                                    }
                                  },
                                  "name": "created_at"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xaa6cd0bf",
                                              "ty": 11
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "SecondsPerLiquidity"
                                    }
                                  },
                                  "name": "seconds_per_liquidity_inside"
                                }
                              ],
                              "name": "Position"
                            }
                          },
                          "root_key": "0xaa6cd0bf",
                          "ty": 13
                        }
                      },
                      "name": "positions"
                    }
                  ],
                  "name": "Positions"
                }
              },
              "name": "positions"
            },
            {
              "layout": {
                "struct": {
                  "fields": [
                    {
                      "layout": {
                        "root": {
                          "layout": {
                            "struct": {
                              "fields": [
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xfe7b1486",
                                              "ty": 11
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "Liquidity"
                                    }
                                  },
                                  "name": "liquidity"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xfe7b1486",
                                              "ty": 11
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "SqrtPrice"
                                    }
                                  },
                                  "name": "sqrt_price"
                                },
                                {
                                  "layout": {
                                    "leaf": {
                                      "key": "0xfe7b1486",
                                      "ty": 12
                                    }
                                  },
                                  "name": "current_tick_index"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "struct": {
                                              "fields": [
                                                {
                                                  "layout": {
                                                    "array": {
                                                      "layout": {
                                                        "leaf": {
                                                          "key": "0xfe7b1486",
                                                          "ty": 9
                                                        }
                                                      },
                                                      "len": 4,
                                                      "offset": "0xfe7b1486"
                                                    }
                                                  },
                                                  "name": "0"
                                                }
                                              ],
                                              "name": "U256"
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "FeeGrowth"
                                    }
                                  },
                                  "name": "fee_growth_global_x"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "struct": {
                                              "fields": [
                                                {
                                                  "layout": {
                                                    "array": {
                                                      "layout": {
                                                        "leaf": {
                                                          "key": "0xfe7b1486",
                                                          "ty": 9
                                                        }
                                                      },
                                                      "len": 4,
                                                      "offset": "0xfe7b1486"
                                                    }
                                                  },
                                                  "name": "0"
                                                }
                                              ],
                                              "name": "U256"
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "FeeGrowth"
                                    }
                                  },
                                  "name": "fee_growth_global_y"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xfe7b1486",
                                              "ty": 11
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "TokenAmount"
                                    }
                                  },
                                  "name": "fee_protocol_token_x"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xfe7b1486",
                                              "ty": 11
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "TokenAmount"
                                    }
                                  },
                                  "name": "fee_protocol_token_y"
                                },
                                {
                                  "layout": {
                                    "leaf": {
                                      "key": "0xfe7b1486",
                                      "ty": 9
                                    }
                                  },
                                  "name": "start_timestamp"
                                },
                                {
                                  "layout": {
                                    "leaf": {
                                      "key": "0xfe7b1486",
                                      "ty": 9
                                    }
                                  },
                                  "name": "last_timestamp"
                                },
                                {
                                  "layout": {
                                    "leaf": {
                                      "key": "0xfe7b1486",
                                      "ty": 2
                                    }
                                  },
                                  "name": "fee_receiver"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xfe7b1486",
                                              "ty": 11
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "SecondsPerLiquidity"
                                    }
                                  },
                                  "name": "seconds_per_liquidity_global"
                                }
                              ],
                              "name": "Pool"
                            }
                          },
                          "root_key": "0xfe7b1486",
                          "ty": 25
                        }
                      },
                      "name": "pools"
                    }
                  ],
                  "name": "Pools"
                }
              },
              "name": "pools"
            },
            {
              "layout": {
                "struct": {
                  "fields": [
                    {
                      "layout": {
                        "root": {
                          "layout": {
                            "leaf": {
                              "key": "0xd41cdba5",
                              "ty": 9
                            }
                          },
                          "root_key": "0xd41cdba5",
                          "ty": 30
                        }
                      },
                      "name": "bitmap"
                    }
                  ],
                  "name": "Tickmap"
                }
              },
              "name": "tickmap"
            },
            {
              "layout": {
                "struct": {
                  "fields": [
                    {
                      "layout": {
                        "root": {
                          "layout": {
                            "struct": {
                              "fields": [
                                {
                                  "layout": {
                                    "leaf": {
                                      "key": "0xbc43a840",
                                      "ty": 12
                                    }
                                  },
                                  "name": "index"
                                },
                                {
                                  "layout": {
                                    "leaf": {
                                      "key": "0xbc43a840",
                                      "ty": 34
                                    }
                                  },
                                  "name": "sign"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xbc43a840",
                                              "ty": 11
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "Liquidity"
                                    }
                                  },
                                  "name": "liquidity_change"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xbc43a840",
                                              "ty": 11
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "Liquidity"
                                    }
                                  },
                                  "name": "liquidity_gross"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xbc43a840",
                                              "ty": 11
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "SqrtPrice"
                                    }
                                  },
                                  "name": "sqrt_price"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "struct": {
                                              "fields": [
                                                {
                                                  "layout": {
                                                    "array": {
                                                      "layout": {
                                                        "leaf": {
                                                          "key": "0xbc43a840",
                                                          "ty": 9
                                                        }
                                                      },
                                                      "len": 4,
                                                      "offset": "0xbc43a840"
                                                    }
                                                  },
                                                  "name": "0"
                                                }
                                              ],
                                              "name": "U256"
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "FeeGrowth"
                                    }
                                  },
                                  "name": "fee_growth_outside_x"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "struct": {
                                              "fields": [
                                                {
                                                  "layout": {
                                                    "array": {
                                                      "layout": {
                                                        "leaf": {
                                                          "key": "0xbc43a840",
                                                          "ty": 9
                                                        }
                                                      },
                                                      "len": 4,
                                                      "offset": "0xbc43a840"
                                                    }
                                                  },
                                                  "name": "0"
                                                }
                                              ],
                                              "name": "U256"
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "FeeGrowth"
                                    }
                                  },
                                  "name": "fee_growth_outside_y"
                                },
                                {
                                  "layout": {
                                    "leaf": {
                                      "key": "0xbc43a840",
                                      "ty": 9
                                    }
                                  },
                                  "name": "seconds_outside"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0xbc43a840",
                                              "ty": 11
                                            }
                                          },
                                          "name": "0"
                                        }
                                      ],
                                      "name": "SecondsPerLiquidity"
                                    }
                                  },
                                  "name": "seconds_per_liquidity_outside"
                                }
                              ],
                              "name": "Tick"
                            }
                          },
                          "root_key": "0xbc43a840",
                          "ty": 35
                        }
                      },
                      "name": "ticks"
                    }
                  ],
                  "name": "Ticks"
                }
              },
              "name": "ticks"
            },
            {
              "layout": {
                "struct": {
                  "fields": [
                    {
                      "layout": {
                        "leaf": {
                          "key": "0x00000000",
                          "ty": 40
                        }
                      },
                      "name": "fee_tiers"
                    }
                  ],
                  "name": "FeeTiers"
                }
              },
              "name": "fee_tiers"
            },
            {
              "layout": {
                "struct": {
                  "fields": [
                    {
                      "layout": {
                        "root": {
                          "layout": {
                            "leaf": {
                              "key": "0x19e555c8",
                              "ty": 10
                            }
                          },
                          "root_key": "0x19e555c8",
                          "ty": 41
                        }
                      },
                      "name": "pool_keys"
                    },
                    {
                      "layout": {
                        "root": {
                          "layout": {
                            "struct": {
                              "fields": [
                                {
                                  "layout": {
                                    "leaf": {
                                      "key": "0x68879322",
                                      "ty": 2
                                    }
                                  },
                                  "name": "token_x"
                                },
                                {
                                  "layout": {
                                    "leaf": {
                                      "key": "0x68879322",
                                      "ty": 2
                                    }
                                  },
                                  "name": "token_y"
                                },
                                {
                                  "layout": {
                                    "struct": {
                                      "fields": [
                                        {
                                          "layout": {
                                            "struct": {
                                              "fields": [
                                                {
                                                  "layout": {
                                                    "leaf": {
                                                      "key": "0x68879322",
                                                      "ty": 9
                                                    }
                                                  },
                                                  "name": "0"
                                                }
                                              ],
                                              "name": "Percentage"
                                            }
                                          },
                                          "name": "fee"
                                        },
                                        {
                                          "layout": {
                                            "leaf": {
                                              "key": "0x68879322",
                                              "ty": 10
                                            }
                                          },
                                          "name": "tick_spacing"
                                        }
                                      ],
                                      "name": "FeeTier"
                                    }
                                  },
                                  "name": "fee_tier"
                                }
                              ],
                              "name": "PoolKey"
                            }
                          },
                          "root_key": "0x68879322",
                          "ty": 44
                        }
                      },
                      "name": "pool_keys_by_index"
                    },
                    {
                      "layout": {
                        "leaf": {
                          "key": "0x00000000",
                          "ty": 10
                        }
                      },
                      "name": "pool_keys_length"
                    }
                  ],
                  "name": "PoolKeys"
                }
              },
              "name": "pool_keys"
            },
            {
              "layout": {
                "struct": {
                  "fields": [
                    {
                      "layout": {
                        "leaf": {
                          "key": "0x00000000",
                          "ty": 2
                        }
                      },
                      "name": "admin"
                    },
                    {
                      "layout": {
                        "struct": {
                          "fields": [
                            {
                              "layout": {
                                "leaf": {
                                  "key": "0x00000000",
                                  "ty": 9
                                }
                              },
                              "name": "0"
                            }
                          ],
                          "name": "Percentage"
                        }
                      },
                      "name": "protocol_fee"
                    }
                  ],
                  "name": "InvariantConfig"
                }
              },
              "name": "config"
            }
          ],
          "name": "Invariant"
        }
      },
      "root_key": "0x00000000",
      "ty": 47
    }
  },
  "types": [
    {
      "id": 0,
      "type": {
        "def": {
          "primitive": "u32"
        }
      }
    },
    {
      "id": 1,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "K",
            "type": 2
          },
          {
            "name": "V",
            "type": 0
          },
          {
            "name": "KeyType",
            "type": 5
          }
        ],
        "path": [
          "ink_storage",
          "lazy",
          "mapping",
          "Mapping"
        ]
      }
    },
    {
      "id": 2,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "type": 3,
                "typeName": "[u8; 32]"
              }
            ]
          }
        },
        "path": [
          "ink_primitives",
          "types",
          "AccountId"
        ]
      }
    },
    {
      "id": 3,
      "type": {
        "def": {
          "array": {
            "len": 32,
            "type": 4
          }
        }
      }
    },
    {
      "id": 4,
      "type": {
        "def": {
          "primitive": "u8"
        }
      }
    },
    {
      "id": 5,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "L",
            "type": 6
          },
          {
            "name": "R",
            "type": 7
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ResolverKey"
        ]
      }
    },
    {
      "id": 6,
      "type": {
        "def": {
          "composite": {}
        },
        "path": [
          "ink_storage_traits",
          "impls",
          "AutoKey"
        ]
      }
    },
    {
      "id": 7,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "ParentKey",
            "type": 8
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ManualKey"
        ]
      }
    },
    {
      "id": 8,
      "type": {
        "def": {
          "tuple": []
        }
      }
    },
    {
      "id": 9,
      "type": {
        "def": {
          "primitive": "u64"
        }
      }
    },
    {
      "id": 10,
      "type": {
        "def": {
          "primitive": "u16"
        }
      }
    },
    {
      "id": 11,
      "type": {
        "def": {
          "primitive": "u128"
        }
      }
    },
    {
      "id": 12,
      "type": {
        "def": {
          "primitive": "i32"
        }
      }
    },
    {
      "id": 13,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "K",
            "type": 14
          },
          {
            "name": "V",
            "type": 15
          },
          {
            "name": "KeyType",
            "type": 23
          }
        ],
        "path": [
          "ink_storage",
          "lazy",
          "mapping",
          "Mapping"
        ]
      }
    },
    {
      "id": 14,
      "type": {
        "def": {
          "tuple": [
            2,
            0
          ]
        }
      }
    },
    {
      "id": 15,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "pool_key",
                "type": 16,
                "typeName": "PoolKey"
              },
              {
                "name": "liquidity",
                "type": 19,
                "typeName": "Liquidity"
              },
              {
                "name": "lower_tick_index",
                "type": 12,
                "typeName": "i32"
              },
              {
                "name": "upper_tick_index",
                "type": 12,
                "typeName": "i32"
              },
              {
                "name": "fee_growth_inside_x",
                "type": 20,
                "typeName": "FeeGrowth"
              },
              {
                "name": "fee_growth_inside_y",
                "type": 20,
                "typeName": "FeeGrowth"
              },
              {
                "name": "last_block_number",
                "type": 9,
                "typeName": "u64"
              },
              {
                "name": "tokens_owed_x",
                "type": 23,
                "typeName": "TokenAmount"
              },
              {
                "name": "tokens_owed_y",
                "type": 23,
                "typeName": "TokenAmount"
              },
              {
                "name": "created_at",
                "type": 9,
                "typeName": "u64"
              },
              {
                "name": "seconds_per_liquidity_inside",
                "type": 22,
                "typeName": "SecondsPerLiquidity"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "storage",
          "position",
          "Position"
        ]
      }
    },
    {
      "id": 16,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "token_x",
                "type": 2,
                "typeName": "AccountId"
              },
              {
                "name": "token_y",
                "type": 2,
                "typeName": "AccountId"
              },
              {
                "name": "fee_tier",
                "type": 17,
                "typeName": "FeeTier"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "storage",
          "pool_key",
          "PoolKey"
        ]
      }
    },
    {
      "id": 17,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "fee",
                "type": 18,
                "typeName": "Percentage"
              },
              {
                "name": "tick_spacing",
                "type": 10,
                "typeName": "u16"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "storage",
          "fee_tier",
          "FeeTier"
        ]
      }
    },
    {
      "id": 18,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "type": 9,
                "typeName": "u64"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "math",
          "types",
          "percentage",
          "Percentage"
        ]
      }
    },
    {
      "id": 19,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "type": 11,
                "typeName": "u128"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "math",
          "types",
          "liquidity",
          "Liquidity"
        ]
      }
    },
    {
      "id": 20,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "type": 21,
                "typeName": "U256"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "math",
          "types",
          "fee_growth",
          "FeeGrowth"
        ]
      }
    },
    {
      "id": 21,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "type": 22,
                "typeName": "[u64; 4]"
              }
            ]
          }
        },
        "path": [
          "decimal",
          "uint",
          "U256"
        ]
      }
    },
    {
      "id": 22,
      "type": {
        "def": {
          "array": {
            "len": 4,
            "type": 9
          }
        }
      }
    },
    {
      "id": 23,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "type": 11,
                "typeName": "u128"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "math",
          "types",
          "token_amount",
          "TokenAmount"
        ]
      }
    },
    {
      "id": 24,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "type": 11,
                "typeName": "u128"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "math",
          "types",
          "seconds_per_liquidity",
          "SecondsPerLiquidity"
        ]
      }
    },
    {
      "id": 23,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "L",
            "type": 6
          },
          {
            "name": "R",
            "type": 24
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ResolverKey"
        ]
      }
    },
    {
      "id": 24,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "ParentKey",
            "type": 8
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ManualKey"
        ]
      }
    },
    {
      "id": 25,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "K",
            "type": 16
          },
          {
            "name": "V",
            "type": 26
          },
          {
            "name": "KeyType",
            "type": 28
          }
        ],
        "path": [
          "ink_storage",
          "lazy",
          "mapping",
          "Mapping"
        ]
      }
    },
    {
      "id": 26,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "liquidity",
                "type": 19,
                "typeName": "Liquidity"
              },
              {
                "name": "sqrt_price",
                "type": 27,
                "typeName": "SqrtPrice"
              },
              {
                "name": "current_tick_index",
                "type": 12,
                "typeName": "i32"
              },
              {
                "name": "fee_growth_global_x",
                "type": 20,
                "typeName": "FeeGrowth"
              },
              {
                "name": "fee_growth_global_y",
                "type": 20,
                "typeName": "FeeGrowth"
              },
              {
                "name": "fee_protocol_token_x",
                "type": 23,
                "typeName": "TokenAmount"
              },
              {
                "name": "fee_protocol_token_y",
                "type": 23,
                "typeName": "TokenAmount"
              },
              {
                "name": "start_timestamp",
                "type": 9,
                "typeName": "u64"
              },
              {
                "name": "last_timestamp",
                "type": 9,
                "typeName": "u64"
              },
              {
                "name": "fee_receiver",
                "type": 2,
                "typeName": "AccountId"
              },
              {
                "name": "seconds_per_liquidity_global",
                "type": 22,
                "typeName": "SecondsPerLiquidity"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "storage",
          "pool",
          "Pool"
        ]
      }
    },
    {
      "id": 27,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "type": 11,
                "typeName": "u128"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "math",
          "types",
          "sqrt_price",
          "SqrtPrice"
        ]
      }
    },
    {
      "id": 28,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "L",
            "type": 6
          },
          {
            "name": "R",
            "type": 29
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ResolverKey"
        ]
      }
    },
    {
      "id": 29,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "ParentKey",
            "type": 8
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ManualKey"
        ]
      }
    },
    {
      "id": 30,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "K",
            "type": 31
          },
          {
            "name": "V",
            "type": 9
          },
          {
            "name": "KeyType",
            "type": 32
          }
        ],
        "path": [
          "ink_storage",
          "lazy",
          "mapping",
          "Mapping"
        ]
      }
    },
    {
      "id": 31,
      "type": {
        "def": {
          "tuple": [
            10,
            16
          ]
        }
      }
    },
    {
      "id": 32,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "K",
            "type": 32
          },
          {
            "name": "V",
            "type": 9
          },
          {
            "name": "KeyType",
            "type": 33
          }
        ],
        "path": [
          "ink_storage",
          "lazy",
          "mapping",
          "Mapping"
        ]
      }
    },
    {
      "id": 32,
      "type": {
        "def": {
          "tuple": [
            10,
            16
          ]
        }
      }
    },
    {
      "id": 33,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "L",
            "type": 6
          },
          {
            "name": "R",
            "type": 33
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ResolverKey"
        ]
      }
    },
    {
      "id": 33,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "ParentKey",
            "type": 8
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ManualKey"
        ]
      }
    },
    {
      "id": 34,
      "type": {
        "def": {
          "primitive": "bool"
        }
      }
    },
    {
      "id": 35,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "K",
            "type": 36
          },
          {
            "name": "V",
            "type": 37
          },
          {
            "name": "KeyType",
            "type": 38
          }
        ],
        "path": [
          "ink_storage",
          "lazy",
          "mapping",
          "Mapping"
        ]
      }
    },
    {
      "id": 36,
      "type": {
        "def": {
          "tuple": [
            16,
            12
          ]
        }
      }
    },
    {
      "id": 37,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "index",
                "type": 12,
                "typeName": "i32"
              },
              {
                "name": "sign",
                "type": 34,
                "typeName": "bool"
              },
              {
                "name": "liquidity_change",
                "type": 19,
                "typeName": "Liquidity"
              },
              {
                "name": "liquidity_gross",
                "type": 19,
                "typeName": "Liquidity"
              },
              {
                "name": "sqrt_price",
                "type": 27,
                "typeName": "SqrtPrice"
              },
              {
                "name": "fee_growth_outside_x",
                "type": 20,
                "typeName": "FeeGrowth"
              },
              {
                "name": "fee_growth_outside_y",
                "type": 20,
                "typeName": "FeeGrowth"
              },
              {
                "name": "seconds_outside",
                "type": 9,
                "typeName": "u64"
              },
              {
                "name": "seconds_per_liquidity_outside",
                "type": 22,
                "typeName": "SecondsPerLiquidity"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "storage",
          "tick",
          "Tick"
        ]
      }
    },
    {
      "id": 38,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "L",
            "type": 6
          },
          {
            "name": "R",
            "type": 39
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ResolverKey"
        ]
      }
    },
    {
      "id": 39,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "ParentKey",
            "type": 8
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ManualKey"
        ]
      }
    },
    {
      "id": 40,
      "type": {
        "def": {
          "sequence": {
            "type": 17
          }
        }
      }
    },
    {
      "id": 41,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "K",
            "type": 16
          },
          {
            "name": "V",
            "type": 10
          },
          {
            "name": "KeyType",
            "type": 42
          }
        ],
        "path": [
          "ink_storage",
          "lazy",
          "mapping",
          "Mapping"
        ]
      }
    },
    {
      "id": 42,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "L",
            "type": 6
          },
          {
            "name": "R",
            "type": 43
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ResolverKey"
        ]
      }
    },
    {
      "id": 43,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "ParentKey",
            "type": 8
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ManualKey"
        ]
      }
    },
    {
      "id": 44,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "K",
            "type": 10
          },
          {
            "name": "V",
            "type": 16
          },
          {
            "name": "KeyType",
            "type": 45
          }
        ],
        "path": [
          "ink_storage",
          "lazy",
          "mapping",
          "Mapping"
        ]
      }
    },
    {
      "id": 45,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "L",
            "type": 6
          },
          {
            "name": "R",
            "type": 46
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ResolverKey"
        ]
      }
    },
    {
      "id": 46,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "ParentKey",
            "type": 8
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ManualKey"
        ]
      }
    },
    {
      "id": 47,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "L",
            "type": 6
          },
          {
            "name": "R",
            "type": 47
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ResolverKey"
        ]
      }
    },
    {
      "id": 47,
      "type": {
        "def": {
          "composite": {}
        },
        "params": [
          {
            "name": "ParentKey",
            "type": 8
          }
        ],
        "path": [
          "ink_storage_traits",
          "impls",
          "ManualKey"
        ]
      }
    },
    {
      "id": 48,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "positions",
                "type": 48,
                "typeName": "<Positions as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<4203942951u32, ()>,>>::Type"
              },
              {
                "name": "pools",
                "type": 49,
                "typeName": "<Pools as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<376105818u32, ()>,>>::Type"
              },
              {
                "name": "tickmap",
                "type": 50,
                "typeName": "<Tickmap as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<494648388u32, ()>,>>::Type"
              },
              {
                "name": "ticks",
                "type": 51,
                "typeName": "<Ticks as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<3714784162u32, ()>,>>::Type"
              },
              {
                "name": "fee_tiers",
                "type": 52,
                "typeName": "<FeeTiers as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<2632500823u32, ()>,>>::Type"
              },
              {
                "name": "pool_keys",
                "type": 53,
                "typeName": "<PoolKeys as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<1198328142u32, ()>,>>::Type"
              },
              {
                "name": "config",
                "type": 54,
                "typeName": "<InvariantConfig as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<3494350023u32, ()>,>>::Type"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "invariant",
          "Invariant"
        ]
      }
    },
    {
      "id": 48,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "positions_length",
                "type": 1,
                "typeName": "<Mapping<AccountId, u32> as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<331954937u32, ()>,>>::Type"
              },
              {
                "name": "positions",
                "type": 13,
                "typeName": "<Mapping<(AccountId, u32), Position> as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<3218107562u32,()>,>>::Type"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "collections",
          "positions",
          "Positions"
        ]
      }
    },
    {
      "id": 49,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "pools",
                "type": 25,
                "typeName": "<Mapping<PoolKey, Pool> as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<2249489406u32, ()>,>>::Type"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "collections",
          "pools",
          "Pools"
        ]
      }
    },
    {
      "id": 50,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "bitmap",
                "type": 30,
                "typeName": "<Mapping<(u16, PoolKey), u64> as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<2782600404u32,()>,>>::Type"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "storage",
          "tickmap",
          "Tickmap"
        ]
      }
    },
    {
      "id": 51,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "ticks",
                "type": 35,
                "typeName": "<Mapping<(PoolKey, i32), Tick> as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<1084769212u32,()>,>>::Type"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "collections",
          "ticks",
          "Ticks"
        ]
      }
    },
    {
      "id": 52,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "fee_tiers",
                "type": 40,
                "typeName": "<Vec<FeeTier> as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<2342165498u32, ()>,>>::Type"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "collections",
          "fee_tiers",
          "FeeTiers"
        ]
      }
    },
    {
      "id": 53,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "pool_keys",
                "type": 41,
                "typeName": "<Mapping<PoolKey, u16> as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<3361072409u32, ()>,>>::Type"
              },
              {
                "name": "pool_keys_by_index",
                "type": 44,
                "typeName": "<Mapping<u16, PoolKey> as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<580093800u32, ()>,>>::Type"
              },
              {
                "name": "pool_keys_length",
                "type": 10,
                "typeName": "<u16 as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<3842874649u32, ()>,>>::Type"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "collections",
          "pool_keys",
          "PoolKeys"
        ]
      }
    },
    {
      "id": 54,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "admin",
                "type": 2,
                "typeName": "<AccountId as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<423649979u32, ()>,>>::Type"
              },
              {
                "name": "protocol_fee",
                "type": 18,
                "typeName": "<Percentage as::ink::storage::traits::AutoStorableHint<::ink::storage::traits::ManualKey<1271871885u32, ()>,>>::Type"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "storage",
          "invariant_config",
          "InvariantConfig"
        ]
      }
    },
    {
      "id": 55,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 8
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 8
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 56,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "index": 1,
                "name": "CouldNotReadInput"
              }
            ]
          }
        },
        "path": [
          "ink_primitives",
          "LangError"
        ]
      }
    },
    {
      "id": 57,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 18
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 18
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 60,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 59
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 59
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 59,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 8
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 60
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 8
          },
          {
            "name": "E",
            "type": 60
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 60,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "index": 0,
                "name": "NotAdmin"
              },
              {
                "index": 1,
                "name": "NotFeeReceiver"
              },
              {
                "index": 2,
                "name": "PoolAlreadyExist"
              },
              {
                "index": 3,
                "name": "PoolNotFound"
              },
              {
                "index": 4,
                "name": "TickAlreadyExist"
              },
              {
                "index": 5,
                "name": "InvalidTickIndexOrTickSpacing"
              },
              {
                "index": 6,
                "name": "PositionNotFound"
              },
              {
                "index": 7,
                "name": "TickNotFound"
              },
              {
                "index": 8,
                "name": "FeeTierNotFound"
              },
              {
                "index": 9,
                "name": "PoolKeyNotFound"
              },
              {
                "index": 10,
                "name": "AmountIsZero"
              },
              {
                "index": 11,
                "name": "WrongLimit"
              },
              {
                "index": 12,
                "name": "PriceLimitReached"
              },
              {
                "index": 13,
                "name": "NoGainSwap"
              },
              {
                "index": 14,
                "name": "InvalidTickSpacing"
              },
              {
                "index": 15,
                "name": "FeeTierAlreadyExist"
              },
              {
                "index": 16,
                "name": "PoolKeyAlreadyExist"
              },
              {
                "index": 17,
                "name": "UnauthorizedFeeReceiver"
              },
              {
                "index": 18,
                "name": "ZeroLiquidity"
              },
              {
                "index": 19,
                "name": "TransferError"
              },
              {
                "index": 20,
                "name": "TokensAreSame"
              },
              {
                "index": 21,
                "name": "AmountUnderMinimumAmountOut"
              },
              {
                "index": 22,
                "name": "InvalidFee"
              },
              {
                "index": 23,
                "name": "NotEmptyTickDeinitialization"
              },
              {
                "index": 24,
                "name": "InvalidInitTick"
              },
              {
                "index": 25,
                "name": "InvalidInitSqrtPrice"
              },
              {
                "index": 26,
                "name": "InvalidSize"
              },
              {
                "index": 27,
                "name": "InvalidTickIndex"
              },
              {
                "index": 28,
                "name": "TickLimitReached"
              },
              {
                "fields": [
                  {
                    "type": 11,
                    "typeName": "u128"
                  },
                  {
                    "type": 11,
                    "typeName": "u128"
                  }
                ],
                "index": 29,
                "name": "AddOverflow"
              },
              {
                "fields": [
                  {
                    "type": 11,
                    "typeName": "u128"
                  },
                  {
                    "type": 11,
                    "typeName": "u128"
                  }
                ],
                "index": 30,
                "name": "SubUnderflow"
              },
              {
                "index": 31,
                "name": "MulOverflow"
              },
              {
                "index": 32,
                "name": "DivByZero"
              },
              {
                "index": 33,
                "name": "WAZEROWithdrawError"
              },
              {
                "index": 34,
                "name": "SetCodeHashError"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "error",
          "InvariantError"
        ]
      }
    },
    {
      "id": 61,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 62
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 62
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 62,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 15
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 60
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 15
          },
          {
            "name": "E",
            "type": 60
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 63,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 15
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 15
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 64,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 65
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 60
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 65
          },
          {
            "name": "E",
            "type": 60
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 65,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "amount_in",
                "type": 23,
                "typeName": "TokenAmount"
              },
              {
                "name": "amount_out",
                "type": 23,
                "typeName": "TokenAmount"
              },
              {
                "name": "start_sqrt_price",
                "type": 27,
                "typeName": "SqrtPrice"
              },
              {
                "name": "target_sqrt_price",
                "type": 27,
                "typeName": "SqrtPrice"
              },
              {
                "name": "fee",
                "type": 23,
                "typeName": "TokenAmount"
              },
              {
                "name": "pool",
                "type": 26,
                "typeName": "Pool"
              },
              {
                "name": "ticks",
                "type": 66,
                "typeName": "Vec<Tick>"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "swap_structs",
          "CalculateSwapResult"
        ]
      }
    },
    {
      "id": 66,
      "type": {
        "def": {
          "sequence": {
            "type": 37
          }
        }
      }
    },
    {
      "id": 67,
      "type": {
        "def": {
          "sequence": {
            "type": 68
          }
        }
      }
    },
    {
      "id": 68,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "pool_key",
                "type": 16,
                "typeName": "PoolKey"
              },
              {
                "name": "x_to_y",
                "type": 34,
                "typeName": "bool"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "swap_structs",
          "SwapHop"
        ]
      }
    },
    {
      "id": 69,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 70
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 70
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 70,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 71
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 60
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 71
          },
          {
            "name": "E",
            "type": 60
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 71,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "fields": [
                  {
                    "type": 71
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "name": "amount_out",
                "type": 21,
                "typeName": "TokenAmount"
              },
              {
                "name": "target_sqrt_price",
                "type": 27,
                "typeName": "SqrtPrice"
              },
              {
                "name": "ticks",
                "type": 66,
                "typeName": "Vec<Tick>"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 71
          },
          {
            "name": "E",
            "type": 57
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 72,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "fields": [
                  {
                    "type": 73
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 73
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "invariant",
          "contracts",
          "swap_structs",
          "QuoteResult"
        ]
      }
    },
    {
      "id": 73,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 21
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 60
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 21
          },
          {
            "name": "E",
            "type": 60
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 74,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 23
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 23
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 75,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 76
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 60
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 76
          },
          {
            "name": "E",
            "type": 60
          }
        }
      }
    },
    {
      "id": 76,
      "type": {
        "def": {
          "tuple": [
            77,
            0
          ]
        }
      }
    },
    {
      "id": 77,
      "type": {
        "def": {
          "sequence": {
            "type": 78
          }
        }
      }
    },
    {
      "id": 78,
      "type": {
        "def": {
          "tuple": [
            15,
            26
          ]
        }
      }
    },
    {
      "id": 79,
      "type": {
        "def": {
          "tuple": [
            15,
            27
          ]
        }
      }
    },
    {
      "id": 80,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 81
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 81
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 80,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 81
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 60
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 81
          },
          {
            "name": "E",
            "type": 60
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 81,
      "type": {
        "def": {
          "tuple": [
            21,
            21
          ]
        }
      }
    },
    {
      "id": 82,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 34
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 34
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 83,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 84
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 84
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 84,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 26
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 60
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 26
          },
          {
            "name": "E",
            "type": 60
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 85,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 27
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 27
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 86,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 87
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 60
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 87
          },
          {
            "name": "E",
            "type": 60
          }
        }
      }
    },
    {
      "id": 87,
      "type": {
        "def": {
          "sequence": {
            "type": 88
          }
        }
      }
    },
    {
      "id": 88,
      "type": {
        "def": {
          "tuple": [
            17,
            26
          ]
        }
      }
    },
    {
      "id": 89,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 90
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 90
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 90,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 37
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 60
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 37
          },
          {
            "name": "E",
            "type": 60
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 91,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 38
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 38
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 92,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 93
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 60
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 93
          },
          {
            "name": "E",
            "type": 60
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 93,
      "type": {
        "def": {
          "tuple": [
            94,
            10
          ]
        }
      }
    },
    {
      "id": 94,
      "type": {
        "def": {
          "sequence": {
            "type": 16
          }
        }
      }
    },
    {
      "id": 95,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 40
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 40
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 96,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 41
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 41
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 97,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 98
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 60
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 98
          },
          {
            "name": "E",
            "type": 60
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 98,
      "type": {
        "def": {
          "tuple": [
            15,
            26,
            37,
            37
          ]
        }
      }
    },
    {
      "id": 99,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 100
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 100
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 100,
      "type": {
        "def": {
          "sequence": {
            "type": 101
          }
        }
      }
    },
    {
      "id": 101,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 101
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 57
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 101
          },
          {
            "name": "E",
            "type": 57
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 101,
      "type": {
        "def": {
          "sequence": {
            "type": 102
          }
        }
      }
    },
    {
      "id": 102,
      "type": {
        "def": {
          "tuple": [
            10,
            9
          ]
        }
      }
    },
    {
      "id": 102,
      "type": {
        "def": {
          "sequence": {
            "type": 12
          }
        }
      }
    },
    {
      "id": 103,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 104
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 104
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 104,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 105
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 60
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 105
          },
          {
            "name": "E",
            "type": 60
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 105,
      "type": {
        "def": {
          "sequence": {
            "type": 106
          }
        }
      }
    },
    {
      "id": 106,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "name": "index",
                "type": 12,
                "typeName": "i32"
              },
              {
                "name": "liquidity_change",
                "type": 19,
                "typeName": "Liquidity"
              },
              {
                "name": "sign",
                "type": 34,
                "typeName": "bool"
              }
            ]
          }
        },
        "path": [
          "invariant",
          "contracts",
          "storage",
          "tick",
          "LiquidityTick"
        ]
      }
    },
    {
      "id": 107,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 0
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 0
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 108,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 109
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 56
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 109
          },
          {
            "name": "E",
            "type": 56
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 109,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "fields": [
                  {
                    "type": 0
                  }
                ],
                "index": 0,
                "name": "Ok"
              },
              {
                "fields": [
                  {
                    "type": 60
                  }
                ],
                "index": 1,
                "name": "Err"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 0
          },
          {
            "name": "E",
            "type": 60
          }
        ],
        "path": [
          "Result"
        ]
      }
    },
    {
      "id": 110,
      "type": {
        "def": {
          "composite": {
            "fields": [
              {
                "type": 3,
                "typeName": "[u8; 32]"
              }
            ]
          }
        },
        "path": [
          "ink_primitives",
          "types",
          "Hash"
        ]
      }
    },
    {
      "id": 111,
      "type": {
        "def": {
          "variant": {
            "variants": [
              {
                "index": 0,
                "name": "None"
              },
              {
                "fields": [
                  {
                    "type": 2
                  }
                ],
                "index": 1,
                "name": "Some"
              }
            ]
          }
        },
        "params": [
          {
            "name": "T",
            "type": 2
          }
        ],
        "path": [
          "Option"
        ]
      }
    },
    {
      "id": 112,
      "type": {
        "def": {
          "variant": {}
        },
        "path": [
          "ink_env",
          "types",
          "NoChainExtension"
        ]
      }
    }
  ],
  "version": 5
}`
