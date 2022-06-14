pragma solidity ^0.8.10;

interface Credit {
    function get_credit_score(address target) external view returns (uint256);

    function add_credit_score(address target, uint256 score) external;

    function slash_credit_score(address target, uint256 score) external;
}

contract deeper {
    address private constant DISPATCH =
        0x000000000000000000000000000000000000000A;
    event score(uint256 indexed);

    function toUint256(bytes memory _bytes)
        internal
        pure
        returns (uint256 value)
    {
        assembly {
            value := mload(add(_bytes, 0x20))
        }
    }

    function get_credit_score(address query_address) public returns (uint256) {
        (bool success, bytes memory x) = DISPATCH.call(
            abi.encodeWithSignature("get_credit_score(address)", query_address)
        );
        require(success, "operation ok");

        uint256 y = toUint256(x);
        emit score(y);

        return y;
    }

    function add_credit_score(address query_address, uint256 num) public {
        (bool success, ) = DISPATCH.call(
            abi.encodeWithSignature(
                "add_credit_score(address,uint256)",
                query_address,
                num
            )
        );
        require(success, "operation ok");
    }

    function slash_credit_score(address query_address, uint256 num) public {
        (bool success, ) = DISPATCH.call(
            abi.encodeWithSignature(
                "slash_credit_score(address,uint256)",
                query_address,
                num
            )
        );
        require(success, "operation ok");
    }
}
